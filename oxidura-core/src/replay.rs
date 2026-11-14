//! `replay()`：同步 poll-to-quiescence 的骨架（ADR-0002、ADR-0003、M1-02）。

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context as TaskContext, Poll, Wake, Waker};

use crate::WorkflowTaskFailedCause::WorkflowWorkerUnhandledFailure;
use crate::command::{Command, CompleteWorkflowExecutionAttrs, FailWorkflowExecutionAttrs};
use crate::context::{WfError, WorkflowContext};
use crate::event::HistoryEvent;
use crate::{Failure, Payload, WorkflowTaskFailedCause};

/// 一轮重放的结果。
#[derive(Debug, PartialEq)]
pub enum ReplayOutcome {
    /// root future 返回 `Pending`，且本轮没有被请求唤醒（真静止）。
    Blocked(Vec<Command>),
    /// root future 返回 `Ready`：workflow 已闭合。
    Completed(Vec<Command>),
}

/// 引擎级失败。workflow 自己的 `Err` 不在这里 —— 它是 `FailWorkflowExecution` 命令。
#[derive(Debug, PartialEq)]
pub enum ReplayTaskError {
    PollBudgetExceeded { limit: usize },
    PendingCommandAtClose,
    WorkflowTaskFailed { cause: WorkflowTaskFailedCause },
}

/// 把一段历史重放一轮，产出本轮命令（未物化）。
///
/// 循环语义：首 poll 无条件；此后仅当 waker 被请求唤醒才重 poll。返回 `Pending` 且本轮未被
/// 唤醒即视为静止（`Blocked`）。`budget_limit` 计的是**总 poll 次数**。
pub fn replay<F, Fut>(
    history: &[HistoryEvent],
    budget_limit: usize,
    entry: F,
) -> Result<ReplayOutcome, ReplayTaskError>
where
    F: FnOnce(WorkflowContext) -> Fut,
    Fut: Future<Output = Result<Payload, WfError>>,
{
    let ctx = WorkflowContext::new(history);
    let inspector = ctx.clone();

    let wake = Arc::new(FlagWaker(AtomicBool::new(false)));
    let waker = Waker::from(Arc::clone(&wake));
    let mut task_cx = TaskContext::from_waker(&waker);

    let mut fut = pin!(entry(ctx));
    let mut polls = 0usize;

    loop {
        if polls == budget_limit {
            return Err(ReplayTaskError::PollBudgetExceeded {
                limit: budget_limit,
            });
        }
        polls += 1;

        // 必须在 poll 前清零：否则一次唤醒会污染后续所有 poll 的判据。
        wake.0.store(false, Ordering::SeqCst);

        let polled = catch_unwind(AssertUnwindSafe(|| fut.as_mut().poll(&mut task_cx)));
        match polled {
            Ok(poll) => match poll {
                Poll::Ready(Ok(result)) => {
                    if inspector.has_pending_commands() {
                        return Err(ReplayTaskError::PendingCommandAtClose);
                    }
                    return Ok(ReplayOutcome::Completed(vec![
                        Command::CompleteWorkflowExecution(Box::new(
                            CompleteWorkflowExecutionAttrs { result },
                        )),
                    ]));
                }
                Poll::Ready(Err(err)) => {
                    if inspector.has_pending_commands() {
                        return Err(ReplayTaskError::PendingCommandAtClose);
                    }
                    return Ok(ReplayOutcome::Completed(vec![
                        Command::FailWorkflowExecution(Box::new(FailWorkflowExecutionAttrs {
                            failure: Failure {
                                message: err.message,
                            },
                        })),
                    ]));
                }
                Poll::Pending if !wake.0.load(Ordering::SeqCst) => {
                    return Ok(ReplayOutcome::Blocked(inspector.take_commands()));
                }
                Poll::Pending => {
                    // 被请求唤醒 → 继续；预算在循环头部检查。
                }
            },
            Err(_) => {
                return Err(ReplayTaskError::WorkflowTaskFailed {
                    cause: WorkflowWorkerUnhandledFailure,
                });
            }
        }
    }
}

/// 只记录「有没有被请求唤醒」的 waker：不调度任何东西，所以仍然不是 executor（ADR-0003）。
struct FlagWaker(AtomicBool);

impl Wake for FlagWaker {
    fn wake(self: Arc<Self>) {
        self.0.store(true, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.store(true, Ordering::SeqCst);
    }
}
