//! M1-02 的第一个红测：`replay` 把「立即返回」的 workflow 变成一条闭合命令。

use chrono::{DateTime, TimeZone, Utc};
use oxidura_core::command::{Command, CompleteWorkflowExecutionAttrs};
use oxidura_core::event::{
    Event, HistoryEvent, WorkflowExecutionStartedAttrs, WorkflowTaskScheduledAttrs,
    WorkflowTaskStartedAttrs,
};
use oxidura_core::{Payload, ReplayOutcome, WfError, replay};
use serde_json::json;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 10, 12, 0, 0).unwrap()
}

#[test]
fn replay_completes_immediately() {
    let t1 = now();

    // replay 边界：直到并包含本轮的 WorkflowTaskStarted（ADR-0010）。
    // 历史里必须有 WorkflowTaskScheduled —— 它是 WorkflowTaskStarted.scheduled_event_id 的锚点。
    let history = vec![
        HistoryEvent {
            event_id: 1,
            event_time: t1,
            attributes: Event::WorkflowExecutionStarted(Box::new(WorkflowExecutionStartedAttrs {
                workflow_type: "hello".to_string(),
                task_queue: "main".to_string(),
                input: json!(null),
                workflow_execution_timeout: None,
                workflow_run_timeout: None,
                workflow_task_timeout: None,
                attempt: 1,
            })),
        },
        HistoryEvent {
            event_id: 2,
            event_time: t1,
            attributes: Event::WorkflowTaskScheduled(Box::new(WorkflowTaskScheduledAttrs {
                task_queue: "main".to_string(),
                start_to_close_timeout: None,
                attempt: 1,
            })),
        },
        HistoryEvent {
            event_id: 3,
            event_time: t1,
            attributes: Event::WorkflowTaskStarted(Box::new(WorkflowTaskStartedAttrs {
                scheduled_event_id: 2,
                identity: "test".to_string(),
            })),
        },
    ];

    let result = replay(&history, 100, |_ctx| async move {
        Ok::<Payload, WfError>(json!("hello"))
    });

    assert_eq!(
        result,
        Ok(ReplayOutcome::Completed(vec![
            Command::CompleteWorkflowExecution(Box::new(CompleteWorkflowExecutionAttrs {
                result: json!("hello"),
            })),
        ])),
        "workflow 返回 Ok(v) 应产出一条 CompleteWorkflowExecution 命令"
    );
}
