//! M1-02：`replay` 的同步驱动。

use std::pin::Pin;
use std::task::{Context, Poll};

use chrono::{DateTime, Duration, TimeZone, Utc};
use oxidura_core::command::{Command, CompleteWorkflowExecutionAttrs, ScheduleActivityTaskAttrs};
use oxidura_core::event::{
    ActivityTaskCompletedAttrs, ActivityTaskStartedAttrs, Event, HistoryEvent,
    WorkflowExecutionStartedAttrs, WorkflowTaskCompletedAttrs, WorkflowTaskScheduledAttrs,
    WorkflowTaskStartedAttrs,
};
use oxidura_core::{Payload, ReplayOutcome, ReplayTaskError, WfError, apply_commands, replay};
use serde_json::json;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 10, 12, 0, 0).unwrap()
}

fn event(event_id: i64, event_time: DateTime<Utc>, attributes: Event) -> HistoryEvent {
    HistoryEvent {
        event_id,
        event_time,
        attributes,
    }
}

fn wf_started(event_id: i64, event_time: DateTime<Utc>) -> HistoryEvent {
    event(
        event_id,
        event_time,
        Event::WorkflowExecutionStarted(Box::new(WorkflowExecutionStartedAttrs {
            workflow_type: "order".to_string(),
            task_queue: "orders".to_string(),
            input: json!(null),
            workflow_execution_timeout: None,
            workflow_run_timeout: None,
            workflow_task_timeout: None,
            attempt: 1,
        })),
    )
}

fn wft_scheduled(event_id: i64, event_time: DateTime<Utc>) -> HistoryEvent {
    event(
        event_id,
        event_time,
        Event::WorkflowTaskScheduled(Box::new(WorkflowTaskScheduledAttrs {
            task_queue: "orders".to_string(),
            start_to_close_timeout: None,
            attempt: 1,
        })),
    )
}

fn wft_started(event_id: i64, event_time: DateTime<Utc>, scheduled_event_id: i64) -> HistoryEvent {
    event(
        event_id,
        event_time,
        Event::WorkflowTaskStarted(Box::new(WorkflowTaskStartedAttrs {
            scheduled_event_id,
            identity: "test".to_string(),
        })),
    )
}

#[test]
fn replay_completes_immediately() {
    let t1 = now();

    // replay 边界：直到并包含本轮的 WorkflowTaskStarted（ADR-0010）。
    let history = vec![
        wf_started(1, t1),
        wft_scheduled(2, t1),
        wft_started(3, t1, 2),
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

#[test]
fn replay_blocks_and_emits_schedule_activity() {
    let t1 = now();
    let history = vec![
        wf_started(1, t1),
        wft_scheduled(2, t1),
        wft_started(3, t1, 2),
    ];

    let outcome = replay(&history, 100, |ctx| async move {
        ctx.execute_activity("charge", json!("order-42")).await?;
        Ok::<Payload, WfError>(json!(null))
    })
    .unwrap();

    let ReplayOutcome::Blocked(commands) = outcome else {
        panic!("没有可消费的历史，应当 Blocked，得到 {outcome:?}");
    };
    assert_eq!(commands.len(), 1, "应当正好登记一条命令");
    let Command::ScheduleActivityTask(scheduled) = &commands[0] else {
        panic!("应当是 ScheduleActivityTask，得到 {:?}", commands[0]);
    };
    assert_eq!(scheduled.activity_id, "0", "activity_id 由每轮 seq 派生");
    assert_eq!(scheduled.activity_type, "charge");
    assert_eq!(scheduled.input, json!("order-42"));
}

#[test]
fn replay_now_comes_from_replay_boundary() {
    let t1 = now();
    let history = vec![
        wf_started(1, t1),
        wft_scheduled(2, t1),
        wft_started(3, t1, 2),
    ];

    let outcome = replay(&history, 100, |ctx| async move {
        ctx.execute_activity("charge", json!({ "now": ctx.now() }))
            .await?;
        Ok::<Payload, WfError>(json!(null))
    })
    .unwrap();

    let ReplayOutcome::Blocked(commands) = outcome else {
        panic!("应当 Blocked，得到 {outcome:?}");
    };
    let Command::ScheduleActivityTask(scheduled) = &commands[0] else {
        panic!("应当是 ScheduleActivityTask");
    };
    assert_eq!(
        scheduled.input,
        json!({ "now": t1 }),
        "ctx.now() 必须来自边界里的 WorkflowTaskStarted，而不是系统时间"
    );
}

#[test]
fn replay_consumes_activity_result() {
    let t1 = now();
    let t2 = t1 + Duration::seconds(30);

    let mut history = vec![
        wf_started(1, t1),
        wft_scheduled(2, t1),
        wft_started(3, t1, 2),
    ];

    // 上一轮的提交：WFT Completed + 命令物化（ADR-0008）。
    history.push(event(
        4,
        t1,
        Event::WorkflowTaskCompleted(Box::new(WorkflowTaskCompletedAttrs {
            scheduled_event_id: 2,
            started_event_id: 3,
            identity: "worker".to_string(),
        })),
    ));
    let command = Command::ScheduleActivityTask(Box::new(ScheduleActivityTaskAttrs {
        activity_id: "0".to_string(),
        activity_type: "charge".to_string(),
        task_queue: "activities".to_string(),
        input: json!("order-42"),
        schedule_to_close_timeout: None,
        schedule_to_start_timeout: None,
        start_to_close_timeout: None,
        heartbeat_timeout: None,
    }));
    history.extend(apply_commands(4, 4, t1, &[command]));

    // 外部世界：活动跑完了（不走 apply_commands）。
    history.push(event(
        6,
        t1,
        Event::ActivityTaskStarted(Box::new(ActivityTaskStartedAttrs {
            scheduled_event_id: 5,
            identity: "activity-worker".to_string(),
            attempt: 1,
            last_failure: None,
        })),
    ));
    history.push(event(
        7,
        t2,
        Event::ActivityTaskCompleted(Box::new(ActivityTaskCompletedAttrs {
            result: json!("charged"),
            scheduled_event_id: 5,
            started_event_id: 6,
            identity: "activity-worker".to_string(),
        })),
    ));

    // 本轮新的 WFT 边界。
    history.push(wft_scheduled(8, t2));
    history.push(wft_started(9, t2, 8));

    let outcome = replay(&history, 100, |ctx| async move {
        let result = ctx.execute_activity("charge", json!("order-42")).await?;
        Ok::<Payload, WfError>(json!({ "result": result }))
    })
    .unwrap();

    let ReplayOutcome::Completed(commands) = outcome else {
        panic!("历史里已有活动结果，应当 Completed，得到 {outcome:?}");
    };
    let Command::CompleteWorkflowExecution(completed) = &commands[0] else {
        panic!("应当是 CompleteWorkflowExecution");
    };
    assert_eq!(completed.result, json!({ "result": "charged" }));
}

#[test]
fn replay_multi_round_activity_round_trip() {
    let t1 = now();
    let t2 = t1 + Duration::seconds(30);

    let mut history = vec![
        wf_started(1, t1),
        wft_scheduled(2, t1),
        wft_started(3, t1, 2),
    ];

    let workflow = |ctx: oxidura_core::WorkflowContext| async move {
        let result = ctx
            .execute_activity("charge", json!({ "now": ctx.now() }))
            .await?;
        Ok::<Payload, WfError>(json!({ "now": ctx.now(), "result": result }))
    };

    // ── 第一轮：没有活动结果 → 登记命令并静止 ──
    let outcome = replay(&history, 100, workflow).unwrap();
    let ReplayOutcome::Blocked(commands) = outcome else {
        panic!("第一轮应当 Blocked，得到 {outcome:?}");
    };
    assert_eq!(commands.len(), 1, "第一轮应当正好登记一条命令");
    let round1_now = {
        let Command::ScheduleActivityTask(scheduled) = &commands[0] else {
            panic!("应当是 ScheduleActivityTask，得到 {:?}", commands[0]);
        };
        scheduled.input["now"].clone()
    };

    // ── 模拟第一轮的提交（ADR-0008），再注入外部世界的事件与新边界 ──
    let wft_completed_id = history.len() as i64 + 1;
    history.push(event(
        wft_completed_id,
        t1,
        Event::WorkflowTaskCompleted(Box::new(WorkflowTaskCompletedAttrs {
            scheduled_event_id: 2,
            started_event_id: 3,
            identity: "worker".to_string(),
        })),
    ));
    history.extend(apply_commands(
        history.len() as i64,
        wft_completed_id,
        t1,
        &commands,
    ));
    let activity_scheduled_id = history.last().unwrap().event_id;

    let activity_started_id = activity_scheduled_id + 1;
    history.push(event(
        activity_started_id,
        t1,
        Event::ActivityTaskStarted(Box::new(ActivityTaskStartedAttrs {
            scheduled_event_id: activity_scheduled_id,
            identity: "activity-worker".to_string(),
            attempt: 1,
            last_failure: None,
        })),
    ));
    history.push(event(
        activity_started_id + 1,
        t2,
        Event::ActivityTaskCompleted(Box::new(ActivityTaskCompletedAttrs {
            result: json!("ok"),
            scheduled_event_id: activity_scheduled_id,
            started_event_id: activity_started_id,
            identity: "activity-worker".to_string(),
        })),
    ));

    let next_scheduled_id = history.len() as i64 + 1;
    history.push(wft_scheduled(next_scheduled_id, t2));
    history.push(wft_started(next_scheduled_id + 1, t2, next_scheduled_id));

    // ── 第二轮：从零重放，结果解析回原来的 await 点 ──
    let outcome = replay(&history, 100, workflow).unwrap();
    let ReplayOutcome::Completed(commands) = outcome else {
        panic!("第二轮应当 Completed，得到 {outcome:?}");
    };
    let Command::CompleteWorkflowExecution(completed) = &commands[0] else {
        panic!("应当是 CompleteWorkflowExecution，得到 {:?}", commands[0]);
    };

    let round2_now = completed.result["now"].clone();
    assert_eq!(round1_now, json!(t1), "第一轮时钟");
    assert_eq!(round2_now, json!(t2), "第二轮时钟");
    assert_ne!(round1_now, round2_now, "两轮的 ctx.now() 必须不同");
    assert_eq!(completed.result["result"], json!("ok"));
}

/// 只唤醒自己、永远不推进的 future。
struct SelfWake;

impl Future for SelfWake {
    type Output = Result<Payload, WfError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

#[test]
fn poll_budget_exceeded() {
    let t1 = now();
    let history = vec![
        wf_started(1, t1),
        wft_scheduled(2, t1),
        wft_started(3, t1, 2),
    ];

    let result = replay(&history, 3, |_ctx| SelfWake);

    assert_eq!(
        result,
        Err(ReplayTaskError::PollBudgetExceeded { limit: 3 }),
        "自唤醒的 future 必须被预算截断，而不是被误判成 Blocked"
    );
}

#[test]
fn replay_emits_fail_workflow_execution_on_workflow_error() {
    let t1 = now();
    let t2 = t1 + Duration::seconds(30);
    let mut history = vec![
        wf_started(1, t1),
        wft_scheduled(2, t1),
        wft_started(3, t1, 2),
    ];

    let wft_completed_id = history.len() as i64 + 1;
    history.push(event(
        wft_completed_id,
        t1,
        Event::WorkflowTaskCompleted(Box::new(WorkflowTaskCompletedAttrs {
            scheduled_event_id: 2,
            started_event_id: 3,
            identity: "worker".to_string(),
        })),
    ));
    let command = Command::ScheduleActivityTask(Box::new(ScheduleActivityTaskAttrs {
        activity_id: "0".to_string(),
        activity_type: "charge".to_string(),
        task_queue: "activities".to_string(),
        input: json!("order-42"),
        schedule_to_close_timeout: None,
        schedule_to_start_timeout: None,
        start_to_close_timeout: None,
        heartbeat_timeout: None,
    }));
    history.extend(apply_commands(
        history.len() as i64,
        wft_completed_id,
        t1,
        &[command],
    ));

    // 外部世界：活动跑完了（不走 apply_commands）。
    history.push(event(
        6,
        t1,
        Event::ActivityTaskStarted(Box::new(ActivityTaskStartedAttrs {
            scheduled_event_id: 5,
            identity: "activity-worker".to_string(),
            attempt: 1,
            last_failure: None,
        })),
    ));
    history.push(event(
        7,
        t2,
        Event::ActivityTaskCompleted(Box::new(ActivityTaskCompletedAttrs {
            result: json!("charged"),
            scheduled_event_id: 5,
            started_event_id: 6,
            identity: "activity-worker".to_string(),
        })),
    ));

    // 本轮新的 WFT 边界。
    history.push(wft_scheduled(8, t2));
    history.push(wft_started(9, t2, 8));

    let outcome = replay(&history, 3, |ctx| async move {
        let _result = ctx.execute_activity("charge", json!("ping")).await?;
        Err(WfError {
            message: "charge failed".to_string(),
        })
    })
    .unwrap();

    let ReplayOutcome::Completed(commands) = outcome else {
        panic!("execution failure 是正常终态，应当 Completed，得到 {outcome:?}");
    };
    let Command::FailWorkflowExecution(failed) = &commands[0] else {
        panic!("应当是 FailWorkflowExecution，得到 {:?}", commands[0]);
    };
    assert_eq!(failed.failure.message, "charge failed");
}
