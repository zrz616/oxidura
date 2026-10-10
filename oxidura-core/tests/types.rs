//! 类型形状：serde 往返不丢字段，且事件名 / 命令名 / 编号与 Temporal 官方一致（ADR-0006、M1-01）。

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use oxidura_core::command::{
    Command, CommandType, CompleteWorkflowExecutionAttrs, FailWorkflowExecutionAttrs,
    RecordMarkerAttrs, ScheduleActivityTaskAttrs, StartTimerAttrs,
};
use oxidura_core::event::{
    ActivityTaskCompletedAttrs, ActivityTaskFailedAttrs, ActivityTaskScheduledAttrs,
    ActivityTaskStartedAttrs, ActivityTaskTimedOutAttrs, Event, EventType, HistoryEvent,
    MarkerRecordedAttrs, TimerFiredAttrs, TimerStartedAttrs, WorkflowExecutionCompletedAttrs,
    WorkflowExecutionFailedAttrs, WorkflowExecutionStartedAttrs, WorkflowExecutionTimedOutAttrs,
    WorkflowTaskCompletedAttrs, WorkflowTaskFailedAttrs, WorkflowTaskScheduledAttrs,
    WorkflowTaskStartedAttrs, WorkflowTaskTimedOutAttrs,
};
use oxidura_core::failure::{Failure, TimeoutType};
use oxidura_core::{Payload, WorkflowExecutionStatus, WorkflowTaskFailedCause};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 10, 12, 0, 0).unwrap()
}

fn payload(text: &str) -> Payload {
    Payload::from(text)
}

fn failure() -> Failure {
    Failure {
        message: "boom".to_string(),
    }
}

fn details() -> BTreeMap<String, Payload> {
    BTreeMap::from([("k".to_string(), payload("v"))])
}

struct EventCase {
    event: Event,
    event_type: EventType,
    number: i16,
    name: &'static str,
}

/// 每个 M1 事件 variant 各一份，字段都取非默认值以暴露往返丢字段。
fn event_samples() -> Vec<EventCase> {
    vec![
        EventCase {
            event: Event::WorkflowExecutionStarted(Box::new(WorkflowExecutionStartedAttrs {
                workflow_type: "order".to_string(),
                task_queue: "orders".to_string(),
                input: payload("in"),
                workflow_execution_timeout: Some(Duration::from_secs(3600)),
                workflow_run_timeout: Some(Duration::from_secs(600)),
                workflow_task_timeout: Some(Duration::from_secs(10)),
                attempt: 1,
            })),
            event_type: EventType::WorkflowExecutionStarted,
            number: 1,
            name: "WORKFLOW_EXECUTION_STARTED",
        },
        EventCase {
            event: Event::WorkflowExecutionCompleted(Box::new(WorkflowExecutionCompletedAttrs {
                result: payload("done"),
                workflow_task_completed_event_id: 7,
            })),
            event_type: EventType::WorkflowExecutionCompleted,
            number: 2,
            name: "WORKFLOW_EXECUTION_COMPLETED",
        },
        EventCase {
            event: Event::WorkflowExecutionFailed(Box::new(WorkflowExecutionFailedAttrs {
                failure: failure(),
                workflow_task_completed_event_id: 7,
            })),
            event_type: EventType::WorkflowExecutionFailed,
            number: 3,
            name: "WORKFLOW_EXECUTION_FAILED",
        },
        EventCase {
            event: Event::WorkflowExecutionTimedOut(Box::new(WorkflowExecutionTimedOutAttrs {
                new_execution_run_id: None,
            })),
            event_type: EventType::WorkflowExecutionTimedOut,
            number: 4,
            name: "WORKFLOW_EXECUTION_TIMED_OUT",
        },
        EventCase {
            event: Event::WorkflowTaskScheduled(Box::new(WorkflowTaskScheduledAttrs {
                task_queue: "orders".to_string(),
                start_to_close_timeout: Some(Duration::from_secs(10)),
                attempt: 1,
            })),
            event_type: EventType::WorkflowTaskScheduled,
            number: 5,
            name: "WORKFLOW_TASK_SCHEDULED",
        },
        EventCase {
            event: Event::WorkflowTaskStarted(Box::new(WorkflowTaskStartedAttrs {
                scheduled_event_id: 5,
                identity: "worker-1".to_string(),
            })),
            event_type: EventType::WorkflowTaskStarted,
            number: 6,
            name: "WORKFLOW_TASK_STARTED",
        },
        EventCase {
            event: Event::WorkflowTaskCompleted(Box::new(WorkflowTaskCompletedAttrs {
                scheduled_event_id: 5,
                started_event_id: 6,
                identity: "worker-1".to_string(),
            })),
            event_type: EventType::WorkflowTaskCompleted,
            number: 7,
            name: "WORKFLOW_TASK_COMPLETED",
        },
        EventCase {
            event: Event::WorkflowTaskFailed(Box::new(WorkflowTaskFailedAttrs {
                scheduled_event_id: 5,
                started_event_id: 6,
                cause: WorkflowTaskFailedCause::NonDeterministicError,
                failure: failure(),
                identity: "worker-1".to_string(),
            })),
            event_type: EventType::WorkflowTaskFailed,
            number: 9,
            name: "WORKFLOW_TASK_FAILED",
        },
        EventCase {
            event: Event::WorkflowTaskTimedOut(Box::new(WorkflowTaskTimedOutAttrs {
                scheduled_event_id: 5,
                started_event_id: 6,
                timeout_type: TimeoutType::StartToClose,
            })),
            event_type: EventType::WorkflowTaskTimedOut,
            number: 8,
            name: "WORKFLOW_TASK_TIMED_OUT",
        },
        EventCase {
            event: Event::ActivityTaskScheduled(Box::new(ActivityTaskScheduledAttrs {
                activity_id: "charge-42".to_string(),
                activity_type: "charge_card".to_string(),
                task_queue: "activities".to_string(),
                input: payload("order-42"),
                schedule_to_close_timeout: Some(Duration::from_secs(120)),
                schedule_to_start_timeout: None,
                start_to_close_timeout: Some(Duration::from_secs(30)),
                heartbeat_timeout: None,
                workflow_task_completed_event_id: 7,
            })),
            event_type: EventType::ActivityTaskScheduled,
            number: 10,
            name: "ACTIVITY_TASK_SCHEDULED",
        },
        EventCase {
            event: Event::ActivityTaskStarted(Box::new(ActivityTaskStartedAttrs {
                scheduled_event_id: 10,
                identity: "worker-1".to_string(),
                attempt: 1,
                last_failure: Some(failure()),
            })),
            event_type: EventType::ActivityTaskStarted,
            number: 11,
            name: "ACTIVITY_TASK_STARTED",
        },
        EventCase {
            event: Event::ActivityTaskCompleted(Box::new(ActivityTaskCompletedAttrs {
                result: payload("charged"),
                scheduled_event_id: 10,
                started_event_id: 11,
                identity: "worker-1".to_string(),
            })),
            event_type: EventType::ActivityTaskCompleted,
            number: 12,
            name: "ACTIVITY_TASK_COMPLETED",
        },
        EventCase {
            event: Event::ActivityTaskFailed(Box::new(ActivityTaskFailedAttrs {
                failure: failure(),
                scheduled_event_id: 10,
                started_event_id: 11,
                identity: "worker-1".to_string(),
            })),
            event_type: EventType::ActivityTaskFailed,
            number: 13,
            name: "ACTIVITY_TASK_FAILED",
        },
        EventCase {
            event: Event::ActivityTaskTimedOut(Box::new(ActivityTaskTimedOutAttrs {
                failure: failure(),
                scheduled_event_id: 10,
                started_event_id: 11,
            })),
            event_type: EventType::ActivityTaskTimedOut,
            number: 14,
            name: "ACTIVITY_TASK_TIMED_OUT",
        },
        EventCase {
            event: Event::TimerStarted(Box::new(TimerStartedAttrs {
                timer_id: "1".to_string(),
                start_to_fire_timeout: Duration::from_secs(30),
                workflow_task_completed_event_id: 7,
            })),
            event_type: EventType::TimerStarted,
            number: 17,
            name: "TIMER_STARTED",
        },
        EventCase {
            event: Event::TimerFired(Box::new(TimerFiredAttrs {
                timer_id: "1".to_string(),
                started_event_id: 17,
            })),
            event_type: EventType::TimerFired,
            number: 18,
            name: "TIMER_FIRED",
        },
        EventCase {
            event: Event::MarkerRecorded(Box::new(MarkerRecordedAttrs {
                marker_name: "side_effect".to_string(),
                details: details(),
                workflow_task_completed_event_id: 7,
                failure: None,
            })),
            event_type: EventType::MarkerRecorded,
            number: 25,
            name: "MARKER_RECORDED",
        },
    ]
}

struct CommandCase {
    command: Command,
    command_type: CommandType,
    number: i16,
    name: &'static str,
}

fn command_samples() -> Vec<CommandCase> {
    vec![
        CommandCase {
            command: Command::ScheduleActivityTask(Box::new(ScheduleActivityTaskAttrs {
                activity_id: "charge-42".to_string(),
                activity_type: "charge_card".to_string(),
                task_queue: "activities".to_string(),
                input: payload("order-42"),
                schedule_to_close_timeout: Some(Duration::from_secs(120)),
                schedule_to_start_timeout: Some(Duration::from_secs(5)),
                start_to_close_timeout: Some(Duration::from_secs(30)),
                heartbeat_timeout: Some(Duration::from_secs(10)),
            })),
            command_type: CommandType::ScheduleActivityTask,
            number: 1,
            name: "SCHEDULE_ACTIVITY_TASK",
        },
        CommandCase {
            command: Command::StartTimer(Box::new(StartTimerAttrs {
                timer_id: "3".to_string(),
                start_to_fire_timeout: Duration::from_secs(5),
            })),
            command_type: CommandType::StartTimer,
            number: 3,
            name: "START_TIMER",
        },
        CommandCase {
            command: Command::CompleteWorkflowExecution(Box::new(CompleteWorkflowExecutionAttrs {
                result: payload("done"),
            })),
            command_type: CommandType::CompleteWorkflowExecution,
            number: 4,
            name: "COMPLETE_WORKFLOW_EXECUTION",
        },
        CommandCase {
            command: Command::FailWorkflowExecution(Box::new(FailWorkflowExecutionAttrs {
                failure: failure(),
            })),
            command_type: CommandType::FailWorkflowExecution,
            number: 5,
            name: "FAIL_WORKFLOW_EXECUTION",
        },
        CommandCase {
            command: Command::RecordMarker(Box::new(RecordMarkerAttrs {
                marker_name: "side_effect".to_string(),
                details: details(),
                failure: Some(failure()),
            })),
            command_type: CommandType::RecordMarker,
            number: 9,
            name: "RECORD_MARKER",
        },
    ]
}

#[test]
fn event_names_numbers_and_types_match_temporal() {
    for case in event_samples() {
        let name = case.name;
        assert_eq!(case.event.event_type(), case.event_type, "{name}");
        assert_eq!(case.event_type.as_i16(), case.number, "{name}");
        assert_eq!(
            EventType::from_i16(case.number),
            Some(case.event_type),
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(case.event_type).unwrap(),
            serde_json::Value::from(name),
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(&case.event).unwrap()["event_type"],
            serde_json::Value::from(name),
            "{name}"
        );
    }
}

#[test]
fn history_event_serde_round_trip_keeps_every_field() {
    for case in event_samples() {
        let event = HistoryEvent {
            event_id: 11,
            event_time: now(),
            attributes: case.event,
        };
        let encoded = serde_json::to_string(&event).unwrap();
        let decoded: HistoryEvent = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, event, "{}", case.name);
    }
}

#[test]
fn command_names_numbers_and_types_match_temporal() {
    for case in command_samples() {
        let name = case.name;
        assert_eq!(case.command.command_type(), case.command_type, "{name}");
        assert_eq!(case.command_type.as_i16(), case.number, "{name}");
        assert_eq!(
            CommandType::from_i16(case.number),
            Some(case.command_type),
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(case.command_type).unwrap(),
            serde_json::Value::from(name),
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(&case.command).unwrap()["command_type"],
            serde_json::Value::from(name),
            "{name}"
        );
    }
}

#[test]
fn command_serde_round_trip_keeps_every_field() {
    for case in command_samples() {
        let encoded = serde_json::to_string(&case.command).unwrap();
        let decoded: Command = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, case.command, "{}", case.name);
    }
}

#[test]
fn execution_status_numbers_match_temporal() {
    let cases = [
        (WorkflowExecutionStatus::Running, 1, "RUNNING"),
        (WorkflowExecutionStatus::Completed, 2, "COMPLETED"),
        (WorkflowExecutionStatus::Failed, 3, "FAILED"),
        (WorkflowExecutionStatus::Canceled, 4, "CANCELED"),
        (WorkflowExecutionStatus::Terminated, 5, "TERMINATED"),
        (
            WorkflowExecutionStatus::ContinuedAsNew,
            6,
            "CONTINUED_AS_NEW",
        ),
        (WorkflowExecutionStatus::TimedOut, 7, "TIMED_OUT"),
    ];

    for (status, number, name) in cases {
        assert_eq!(status.as_i16(), number, "{name}");
        assert_eq!(
            WorkflowExecutionStatus::from_i16(number),
            Some(status),
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(status).unwrap(),
            serde_json::Value::from(name),
            "{name}"
        );
    }
    assert_eq!(WorkflowExecutionStatus::from_i16(0), None);
    assert_eq!(EventType::from_i16(0), None);
    assert_eq!(CommandType::from_i16(0), None);
}

#[test]
fn timeout_type_numbers_match_temporal() {
    let cases = [
        (TimeoutType::StartToClose, 1),
        (TimeoutType::ScheduleToStart, 2),
        (TimeoutType::ScheduleToClose, 3),
        (TimeoutType::Heartbeat, 4),
    ];
    for (timeout_type, number) in cases {
        assert_eq!(timeout_type.as_i16(), number);
        assert_eq!(TimeoutType::from_i16(number), Some(timeout_type));
    }
    assert_eq!(TimeoutType::from_i16(0), None);
}
