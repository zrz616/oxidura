//! `replay()`：同步 poll-to-quiescence 的骨架（ADR-0002、ADR-0003、M1-02）。

use std::pin::pin;
use std::task::{Context as TaskContext, Poll, Waker};

use crate::command::{Command, CompleteWorkflowExecutionAttrs};
use crate::context::{WfError, WorkflowContext};
use crate::event::HistoryEvent;
use crate::{Payload, WorkflowTaskFailedCause};

/// 一轮重放的结果。
///
/// `Blocked` / `Completed` 是 **poll 循环** 的性质，不是命令的性质 —— 所以它必须在这里，
/// 不能靠「有闭合命令」反推（P5 的 `wait_signal` 会造出「Blocked 且 0 命令」）。
#[derive(Debug, PartialEq)]
pub enum ReplayOutcome {
    /// root future 返回 `Pending`，且本轮没有被请求唤醒（真静止）。
    Blocked(Vec<Command>),
    /// root future 返回 `Ready`：workflow 已闭合。
    ///
    /// 不变式：这里的命令应恰好是那一条闭合命令（`CompleteWorkflowExecution` 或
    /// `FailWorkflowExecution`），否则就该是 `PendingCommandAtClose`。
    Completed(Vec<Command>),
}

/// 引擎级失败。
///
/// workflow 自己返回的 `Err` 不在这里 —— 它会变成一条 `FailWorkflowExecution` 命令
/// （ADR-0007 的第一类出口：execution failure 是正常终态，不是引擎错误）。
#[derive(Debug, PartialEq)]
pub enum ReplayTaskError {
    /// 重 poll 次数触界：典型诱因是「自己唤醒自己」的 future。
    PollBudgetExceeded { limit: usize },
    /// root future 已 `Ready`，但命令缓冲里仍有未兑现的命令 —— 不静默丢弃。
    PendingCommandAtClose,
    /// workflow 里 panic：翻译成可重试的 workflow-task failure（ADR-0007）。
    WorkflowTaskFailed { cause: WorkflowTaskFailedCause },
}

/// 把一段历史重放一轮，产出本轮命令（未物化；物化是 `apply_commands` 的事）。
///
/// `history` 必须是 **replay 边界**：直到并包含本轮 `WorkflowTaskStarted` —— `ctx.now()`
/// 取的就是这条事件的 `event_time`（ADR-0010）。
///
/// TODO(M1-02)：poll 循环（首 poll 无条件，此后仅在被请求唤醒时重 poll）、预算、命令缓冲。
pub fn replay<F, Fut>(
    history: &[HistoryEvent],
    budget_limit: usize,
    entry: F,
) -> Result<ReplayOutcome, ReplayTaskError>
where
    F: FnOnce(WorkflowContext) -> Fut,
    Fut: Future<Output = Result<Payload, WfError>>,
{
    let _ = (history, budget_limit);

    let mut fut = pin!(entry(WorkflowContext::new()));
    let mut task_cx = TaskContext::from_waker(Waker::noop());

    match fut.as_mut().poll(&mut task_cx) {
        Poll::Ready(Ok(result)) => Ok(ReplayOutcome::Completed(vec![
            Command::CompleteWorkflowExecution(Box::new(CompleteWorkflowExecutionAttrs { result })),
        ])),
        Poll::Ready(Err(_)) => {
            todo!("M1-02：workflow 返回 Err → FailWorkflowExecution 命令")
        }
        // Pending 之后做什么还没有测试规定 —— 不猜。自唤醒应报预算、等 activity 应带 1 条命令，
        // 两者的正确答案现在都还不存在。
        Poll::Pending => todo!("M1-02：Pending → Blocked（需要 wake 判据与命令缓冲）"),
    }
}
