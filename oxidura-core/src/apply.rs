//! `Command` → `HistoryEvent` 的物化（ADR-0008）。

use chrono::{DateTime, Utc};

use crate::command::Command;
use crate::event::{
    ActivityTaskScheduledAttrs, Event, HistoryEvent, MarkerRecordedAttrs, TimerStartedAttrs,
    WorkflowExecutionCompletedAttrs, WorkflowExecutionFailedAttrs,
};

/// 把一轮重放产出的 `commands` 物化成本轮要追加的事件序列（ADR-0008）。
///
/// * `history_len`：追加前历史的事件条数。产出事件的 `event_id` 从 `history_len + 1` 起连续分配
///   （与 `executions.next_event_id` 对齐）。
/// * `now`：产出事件的 `event_time`（重放的确定性时钟，ADR-0010）。
/// * `wft_completed_event_id`：本轮 `WorkflowTaskCompleted` 的事件号，回填进每个「命令派生」事件的
///   `workflow_task_completed_event_id`。
///
/// 纯函数：无 I/O、无校验（命令合法性由 `ctx` 在产出时保证，ADR-0014）。它**只**处理命令派生的事件；
/// 外部世界注入的事件（`TimerFired`、`ActivityTaskCompleted`、`WorkflowExecutionSignaled`）不走这里。
///
/// M1 命令子集产出的都是「派生新事件」，只带 `workflow_task_completed_event_id` 回指；`scheduled_event_id`
/// 型回指（命令作用在既有事件上，如取消活动）随 P4 的 cancel 命令引入。
pub fn apply_commands(
    history_len: i64,
    wft_completed_event_id: i64,
    now: DateTime<Utc>,
    commands: &[Command],
) -> Vec<HistoryEvent> {
    commands
        .iter()
        .enumerate()
        .map(|(offset, command)| HistoryEvent {
            event_id: history_len + 1 + offset as i64,
            event_time: now,
            attributes: materialize(command, wft_completed_event_id),
        })
        .collect()
}

/// 单条命令 → 单条派生事件，并填好 `workflow_task_completed_event_id` 回指。
fn materialize(command: &Command, wft_completed_event_id: i64) -> Event {
    match command {
        Command::ScheduleActivityTask(attrs) => {
            Event::ActivityTaskScheduled(Box::new(ActivityTaskScheduledAttrs {
                activity_id: attrs.activity_id.clone(),
                activity_type: attrs.activity_type.clone(),
                task_queue: attrs.task_queue.clone(),
                input: attrs.input.clone(),
                schedule_to_close_timeout: attrs.schedule_to_close_timeout,
                schedule_to_start_timeout: attrs.schedule_to_start_timeout,
                start_to_close_timeout: attrs.start_to_close_timeout,
                heartbeat_timeout: attrs.heartbeat_timeout,
                workflow_task_completed_event_id: wft_completed_event_id,
            }))
        }
        Command::StartTimer(attrs) => Event::TimerStarted(Box::new(TimerStartedAttrs {
            timer_id: attrs.timer_id.clone(),
            start_to_fire_timeout: attrs.start_to_fire_timeout,
            workflow_task_completed_event_id: wft_completed_event_id,
        })),
        Command::RecordMarker(attrs) => Event::MarkerRecorded(Box::new(MarkerRecordedAttrs {
            marker_name: attrs.marker_name.clone(),
            details: attrs.details.clone(),
            workflow_task_completed_event_id: wft_completed_event_id,
            failure: attrs.failure.clone(),
        })),
        Command::CompleteWorkflowExecution(attrs) => {
            Event::WorkflowExecutionCompleted(Box::new(WorkflowExecutionCompletedAttrs {
                result: attrs.result.clone(),
                workflow_task_completed_event_id: wft_completed_event_id,
            }))
        }
        Command::FailWorkflowExecution(attrs) => {
            Event::WorkflowExecutionFailed(Box::new(WorkflowExecutionFailedAttrs {
                failure: attrs.failure.clone(),
                workflow_task_completed_event_id: wft_completed_event_id,
            }))
        }
    }
}
