//! `apply_commands_*`：命令 → 事件物化与回指字段（ADR-0008、M1-01）。

use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use oxidura_core::Payload;
use oxidura_core::command::{
    Command, CompleteWorkflowExecutionAttrs, FailWorkflowExecutionAttrs, RecordMarkerAttrs,
    ScheduleActivityTaskAttrs, StartTimerAttrs,
};
use oxidura_core::event::Event;
use oxidura_core::failure::Failure;
use oxidura_core::{CommandType, apply_commands};

const HISTORY_LEN: i64 = 10;
const WFT_COMPLETED: i64 = 9;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 10, 12, 0, 0).unwrap()
}

fn payload(text: &str) -> Payload {
    Payload::from(text)
}

fn schedule_command(activity_id: &str) -> Command {
    Command::ScheduleActivityTask(Box::new(ScheduleActivityTaskAttrs {
        activity_id: activity_id.to_string(),
        activity_type: "charge_card".to_string(),
        task_queue: "activities".to_string(),
        input: payload("order-42"),
        schedule_to_close_timeout: Some(Duration::from_secs(120)),
        schedule_to_start_timeout: None,
        start_to_close_timeout: Some(Duration::from_secs(30)),
        heartbeat_timeout: None,
    }))
}

#[test]
fn apply_commands_schedule_activity_copies_attributes_and_backreference() {
    let events = apply_commands(
        HISTORY_LEN,
        WFT_COMPLETED,
        now(),
        &[schedule_command("charge-42")],
    );

    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.event_id, HISTORY_LEN + 1);
    assert_eq!(event.event_time, now());

    let Event::ActivityTaskScheduled(attrs) = &event.attributes else {
        panic!("expected ActivityTaskScheduled, got {:?}", event.attributes);
    };
    assert_eq!(attrs.activity_id, "charge-42");
    assert_eq!(attrs.activity_type, "charge_card");
    assert_eq!(attrs.task_queue, "activities");
    assert_eq!(attrs.input, payload("order-42"));
    assert_eq!(
        attrs.schedule_to_close_timeout,
        Some(Duration::from_secs(120))
    );
    assert_eq!(attrs.start_to_close_timeout, Some(Duration::from_secs(30)));
    assert_eq!(attrs.schedule_to_start_timeout, None);
    assert_eq!(attrs.heartbeat_timeout, None);
    assert_eq!(attrs.workflow_task_completed_event_id, WFT_COMPLETED);
}

#[test]
fn apply_commands_start_timer_and_record_marker_fill_backreference() {
    let marker_details = std::collections::BTreeMap::from([("k".to_string(), payload("v"))]);
    let commands = vec![
        Command::StartTimer(Box::new(StartTimerAttrs {
            timer_id: "3".to_string(),
            start_to_fire_timeout: Duration::from_secs(5),
        })),
        Command::RecordMarker(Box::new(RecordMarkerAttrs {
            marker_name: "side_effect".to_string(),
            details: marker_details.clone(),
            failure: None,
        })),
    ];

    let events = apply_commands(HISTORY_LEN, WFT_COMPLETED, now(), &commands);
    assert_eq!(events.len(), 2);

    let Event::TimerStarted(timer) = &events[0].attributes else {
        panic!("expected TimerStarted");
    };
    assert_eq!(timer.timer_id, "3");
    assert_eq!(timer.start_to_fire_timeout, Duration::from_secs(5));
    assert_eq!(timer.workflow_task_completed_event_id, WFT_COMPLETED);

    let Event::MarkerRecorded(marker) = &events[1].attributes else {
        panic!("expected MarkerRecorded");
    };
    assert_eq!(marker.marker_name, "side_effect");
    assert_eq!(marker.details, marker_details);
    assert_eq!(marker.failure, None);
    assert_eq!(marker.workflow_task_completed_event_id, WFT_COMPLETED);
}

#[test]
fn apply_commands_complete_and_fail_workflow_fill_backreference() {
    let failure = Failure {
        message: "boom".to_string(),
    };
    let commands = vec![
        Command::CompleteWorkflowExecution(Box::new(CompleteWorkflowExecutionAttrs {
            result: payload("done"),
        })),
        Command::FailWorkflowExecution(Box::new(FailWorkflowExecutionAttrs {
            failure: failure.clone(),
        })),
    ];

    let events = apply_commands(HISTORY_LEN, WFT_COMPLETED, now(), &commands);
    assert_eq!(events.len(), 2);

    let Event::WorkflowExecutionCompleted(done) = &events[0].attributes else {
        panic!("expected WorkflowExecutionCompleted");
    };
    assert_eq!(done.result, payload("done"));
    assert_eq!(done.workflow_task_completed_event_id, WFT_COMPLETED);

    let Event::WorkflowExecutionFailed(failed) = &events[1].attributes else {
        panic!("expected WorkflowExecutionFailed");
    };
    assert_eq!(failed.failure, failure);
    assert_eq!(failed.workflow_task_completed_event_id, WFT_COMPLETED);
}

#[test]
fn apply_commands_assigns_sequential_ids_in_command_order() {
    let commands = vec![
        schedule_command("a"),
        schedule_command("b"),
        schedule_command("c"),
    ];

    let events = apply_commands(HISTORY_LEN, WFT_COMPLETED, now(), &commands);

    let ids: Vec<i64> = events.iter().map(|event| event.event_id).collect();
    assert_eq!(ids, vec![HISTORY_LEN + 1, HISTORY_LEN + 2, HISTORY_LEN + 3]);

    let activity_ids: Vec<&str> = events
        .iter()
        .map(|event| match &event.attributes {
            Event::ActivityTaskScheduled(attrs) => attrs.activity_id.as_str(),
            other => panic!("expected ActivityTaskScheduled, got {other:?}"),
        })
        .collect();
    assert_eq!(activity_ids, vec!["a", "b", "c"]);
    assert_eq!(events[0].attributes.event_type().as_i16(), 10);
}

#[test]
fn apply_commands_empty_produces_no_events() {
    assert!(apply_commands(HISTORY_LEN, WFT_COMPLETED, now(), &[]).is_empty());
}

#[test]
fn apply_commands_materializes_one_event_per_command_type() {
    let commands = vec![
        schedule_command("a"),
        Command::StartTimer(Box::new(StartTimerAttrs {
            timer_id: "1".to_string(),
            start_to_fire_timeout: Duration::from_secs(1),
        })),
        Command::RecordMarker(Box::new(RecordMarkerAttrs {
            marker_name: "m".to_string(),
            details: Default::default(),
            failure: None,
        })),
        Command::CompleteWorkflowExecution(Box::new(CompleteWorkflowExecutionAttrs {
            result: payload("ok"),
        })),
    ];

    let events = apply_commands(HISTORY_LEN, WFT_COMPLETED, now(), &commands);
    let types: Vec<CommandType> = commands.iter().map(Command::command_type).collect();
    assert_eq!(
        types,
        vec![
            CommandType::ScheduleActivityTask,
            CommandType::StartTimer,
            CommandType::RecordMarker,
            CommandType::CompleteWorkflowExecution,
        ]
    );

    // 命令与派生事件一一对应、顺序不变。
    let expected_event_numbers = [10_i16, 17, 25, 2];
    let actual_event_numbers: Vec<i16> = events
        .iter()
        .map(|event| event.attributes.event_type().as_i16())
        .collect();
    assert_eq!(actual_event_numbers, expected_event_numbers);
}
