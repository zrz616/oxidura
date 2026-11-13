//! `WorkflowContext` 与 workflow 作者的错误类型（ADR-0009）。

use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context as TaskContext, Poll};

use chrono::{DateTime, Utc};

use crate::command::ScheduleActivityTaskAttrs;
use crate::event::{Event, HistoryEvent};
use crate::{Command, Payload};

/// workflow 作者返回的错误。ADR-0009 定名 `WfError`。
#[derive(Debug, Clone, PartialEq)]
pub struct WfError {
    pub message: String,
}

/// 工作流代码手里的句柄；按值传、廉价 clone、`!Send`（ADR-0003）。
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    inner: Rc<RefCell<Inner>>,
}

/// 一轮重放的内部状态。
#[derive(Debug)]
pub(crate) struct Inner {
    history: Rc<[HistoryEvent]>,
    cursor: usize,
    commands: Vec<Command>,
    seq: usize,
    now: DateTime<Utc>,
}

impl WorkflowContext {
    pub(crate) fn new(history: &[HistoryEvent]) -> Self {
        // ctx.now() = replay 边界里最后一条 WorkflowTaskStarted 的时间（ADR-0010）。
        let now = history
            .iter()
            .rev()
            .find_map(|event| match &event.attributes {
                Event::WorkflowTaskStarted(_) => Some(event.event_time),
                _ => None,
            })
            .expect("replay 边界必须包含 WorkflowTaskStarted（ADR-0010）");

        Self {
            inner: Rc::new(RefCell::new(Inner {
                history: Rc::from(history),
                cursor: 0,
                commands: Vec::new(),
                seq: 0,
                now,
            })),
        }
    }

    /// 确定性时钟 —— 不是系统时间（ADR-0010）。
    pub fn now(&self) -> DateTime<Utc> {
        self.inner.borrow().now
    }

    /// 调度一个活动。
    ///
    /// 历史里有结果 → 立刻返回；没有 → 登记一条 `ScheduleActivityTask` 并 `Pending`。
    pub fn execute_activity(&self, activity_type: &str, input: Payload) -> ActivityFuture {
        let activity_id = {
            let mut inner = self.inner.borrow_mut();
            let activity_id = inner.seq.to_string();
            inner.seq += 1;
            activity_id
        };

        ActivityFuture {
            ctx: self.clone(),
            activity_id,
            activity_type: activity_type.to_string(),
            input,
        }
    }

    /// 本轮是否有未决命令 —— 直接读字段，不克隆缓冲。
    pub(crate) fn has_pending_commands(&self) -> bool {
        !self.inner.borrow().commands.is_empty()
    }

    /// 取走本轮命令（`Blocked` 时交出；交出后 ctx 不再被使用）。
    pub(crate) fn take_commands(&self) -> Vec<Command> {
        std::mem::take(&mut self.inner.borrow_mut().commands)
    }
}

impl Inner {
    /// 游标前进到第一条「会被 await 消费」的事件上。
    fn skip_to_consumable(&mut self) {
        while let Some(event) = self.history.get(self.cursor) {
            if is_skipped(&event.attributes) {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    /// 取下一条可消费事件并推进游标。
    fn next_consumable_event(&mut self) -> Option<HistoryEvent> {
        self.skip_to_consumable();
        let event = self.history.get(self.cursor).cloned();
        if event.is_some() {
            self.cursor += 1;
        }
        event
    }

    fn has_activity_command(&self, activity_id: &str) -> bool {
        self.commands.iter().any(|command| {
            matches!(command, Command::ScheduleActivityTask(attrs) if attrs.activity_id == activity_id)
        })
    }

    fn register_activity(&mut self, activity_id: &str, activity_type: &str, input: Payload) {
        self.commands.push(Command::ScheduleActivityTask(Box::new(
            ScheduleActivityTaskAttrs {
                activity_id: activity_id.to_string(),
                activity_type: activity_type.to_string(),
                // 活动的 task queue 属于 ActivityOptions（P4）；M1 先写死。
                task_queue: "activities".to_string(),
                input,
                schedule_to_close_timeout: None,
                schedule_to_start_timeout: None,
                start_to_close_timeout: None,
                heartbeat_timeout: None,
            },
        )));
    }
}

/// 这些事件**没有任何 await 点会消费它们** —— 游标经过时直接跨过。
///
/// 判据不是「谁写进历史的」（`ActivityTaskScheduled` 同样是引擎物化的），而是：
/// **工作流代码里有没有一个 `.await` 会停下来等它。**
///
/// - `ActivityTaskScheduled` → 要**消费**：活动 await 用它核对 `activity_id`；
/// - `ActivityTaskStarted` → 没人等 → 跳过；
/// - `ActivityTaskCompleted` → 要**消费**：拿结果。
///
/// WFT 三件套是引擎自己的生命周期；`WorkflowExecutionStarted` 是启动引导。都没有 await 等。
///
/// 只列当前测试逼出来的那几种；ADR-0010 的白名单还有 `WorkflowTaskFailed` /
/// `WorkflowTaskTimedOut`，等有测试覆盖时再加（谓词化归 M1-03）。
fn is_skipped(event: &Event) -> bool {
    matches!(
        event,
        Event::WorkflowExecutionStarted(_)
            | Event::WorkflowTaskScheduled(_)
            | Event::WorkflowTaskStarted(_)
            | Event::WorkflowTaskCompleted(_)
            | Event::ActivityTaskStarted(_)
    )
}

/// `ctx.execute_activity(...)` 返回的 future。
#[derive(Debug)]
pub struct ActivityFuture {
    ctx: WorkflowContext,
    activity_id: String,
    activity_type: String,
    input: Payload,
}

impl Future for ActivityFuture {
    type Output = Result<Payload, WfError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();
        let mut inner = me.ctx.inner.borrow_mut();

        match inner.next_consumable_event() {
            Some(HistoryEvent {
                event_id,
                attributes: Event::ActivityTaskScheduled(scheduled),
                ..
            }) if scheduled.activity_id == me.activity_id => match inner.next_consumable_event() {
                Some(HistoryEvent {
                    attributes: Event::ActivityTaskCompleted(completed),
                    ..
                }) if completed.scheduled_event_id == event_id => Poll::Ready(Ok(completed.result)),
                other => unimplemented!("M1-02 只处理成功的活动往返：{other:?}"),
            },
            // 没有可消费的历史 → 登记命令（幂等）并停在 Pending。
            _ => {
                if !inner.has_activity_command(&me.activity_id) {
                    inner.register_activity(&me.activity_id, &me.activity_type, me.input.clone());
                }
                Poll::Pending
            }
        }
    }
}
