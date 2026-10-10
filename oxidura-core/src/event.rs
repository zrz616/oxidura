//! 事件形状：`HistoryEvent` envelope、`Event` oneof，以及事件的官方数字编号（ADR-0005、ADR-0006）。

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Payload;
use crate::failure::{Failure, TimeoutType};

/// 一条历史事件 = `events` 表的一行（ADR-0006）。
///
/// envelope 只留 `event_id` / `event_time` / `attributes`：Temporal 的 `version` / `task_id` /
/// `worker_may_ignore` 等随 D1 一起去掉。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEvent {
    /// 从 1 起单调（Temporal 原义），由 `executions.next_event_id` 分配。
    pub event_id: i64,
    pub event_time: DateTime<Utc>,
    pub attributes: Event,
}

impl HistoryEvent {
    /// 这条事件的官方编号（将来是 `events.event_type SMALLINT`）。
    pub fn event_type(&self) -> EventType {
        self.attributes.event_type()
    }
}

/// 事件属性 oneof；serde 的内部 tag **就是 Temporal 的事件名**（ADR-0006）。
///
/// 每个 variant 用 `Box` 包住属性结构体，避免单个 enum 被最大 variant 撑大
/// （workspace 的 `clippy::large_enum_variant` 兜底）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    WorkflowExecutionStarted(Box<WorkflowExecutionStartedAttrs>),
    WorkflowExecutionCompleted(Box<WorkflowExecutionCompletedAttrs>),
    WorkflowExecutionFailed(Box<WorkflowExecutionFailedAttrs>),
    WorkflowExecutionTimedOut(Box<WorkflowExecutionTimedOutAttrs>),
    WorkflowTaskScheduled(Box<WorkflowTaskScheduledAttrs>),
    WorkflowTaskStarted(Box<WorkflowTaskStartedAttrs>),
    WorkflowTaskCompleted(Box<WorkflowTaskCompletedAttrs>),
    WorkflowTaskFailed(Box<WorkflowTaskFailedAttrs>),
    WorkflowTaskTimedOut(Box<WorkflowTaskTimedOutAttrs>),
    ActivityTaskScheduled(Box<ActivityTaskScheduledAttrs>),
    ActivityTaskStarted(Box<ActivityTaskStartedAttrs>),
    ActivityTaskCompleted(Box<ActivityTaskCompletedAttrs>),
    ActivityTaskFailed(Box<ActivityTaskFailedAttrs>),
    ActivityTaskTimedOut(Box<ActivityTaskTimedOutAttrs>),
    TimerStarted(Box<TimerStartedAttrs>),
    TimerFired(Box<TimerFiredAttrs>),
    MarkerRecorded(Box<MarkerRecordedAttrs>),
}

impl Event {
    /// 事件名对应的官方编号。
    pub fn event_type(&self) -> EventType {
        match self {
            Self::WorkflowExecutionStarted(_) => EventType::WorkflowExecutionStarted,
            Self::WorkflowExecutionCompleted(_) => EventType::WorkflowExecutionCompleted,
            Self::WorkflowExecutionFailed(_) => EventType::WorkflowExecutionFailed,
            Self::WorkflowExecutionTimedOut(_) => EventType::WorkflowExecutionTimedOut,
            Self::WorkflowTaskScheduled(_) => EventType::WorkflowTaskScheduled,
            Self::WorkflowTaskStarted(_) => EventType::WorkflowTaskStarted,
            Self::WorkflowTaskCompleted(_) => EventType::WorkflowTaskCompleted,
            Self::WorkflowTaskFailed(_) => EventType::WorkflowTaskFailed,
            Self::WorkflowTaskTimedOut(_) => EventType::WorkflowTaskTimedOut,
            Self::ActivityTaskScheduled(_) => EventType::ActivityTaskScheduled,
            Self::ActivityTaskStarted(_) => EventType::ActivityTaskStarted,
            Self::ActivityTaskCompleted(_) => EventType::ActivityTaskCompleted,
            Self::ActivityTaskFailed(_) => EventType::ActivityTaskFailed,
            Self::ActivityTaskTimedOut(_) => EventType::ActivityTaskTimedOut,
            Self::TimerStarted(_) => EventType::TimerStarted,
            Self::TimerFired(_) => EventType::TimerFired,
            Self::MarkerRecorded(_) => EventType::MarkerRecorded,
        }
    }
}

/// `WorkflowExecutionStarted` 的属性（字段按需裁剪，名字照抄 Temporal）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionStartedAttrs {
    pub workflow_type: String,
    pub task_queue: String,
    pub input: Payload,
    pub workflow_execution_timeout: Option<Duration>,
    pub workflow_run_timeout: Option<Duration>,
    pub workflow_task_timeout: Option<Duration>,
    pub attempt: i32,
}

/// `WorkflowExecutionCompleted` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionCompletedAttrs {
    pub result: Payload,
    pub workflow_task_completed_event_id: i64,
}

/// `WorkflowExecutionFailed` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionFailedAttrs {
    pub failure: Failure,
    pub workflow_task_completed_event_id: i64,
}

/// `WorkflowExecutionTimedOut` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowExecutionTimedOutAttrs {
    /// 重试链上的下一个 run；M1 不重试 workflow execution，恒为 `None`。
    pub new_execution_run_id: Option<String>,
}

/// `WorkflowTaskScheduled` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowTaskScheduledAttrs {
    pub task_queue: String,
    pub start_to_close_timeout: Option<Duration>,
    pub attempt: i32,
}

/// `WorkflowTaskStarted` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowTaskStartedAttrs {
    pub scheduled_event_id: i64,
    pub identity: String,
}

/// `WorkflowTaskCompleted` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowTaskCompletedAttrs {
    pub scheduled_event_id: i64,
    pub started_event_id: i64,
    pub identity: String,
}

/// `WorkflowTaskFailed` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowTaskFailedAttrs {
    pub scheduled_event_id: i64,
    pub started_event_id: i64,
    pub cause: WorkflowTaskFailedCause,
    pub failure: Failure,
    pub identity: String,
}

/// `WorkflowTaskTimedOut` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowTaskTimedOutAttrs {
    pub scheduled_event_id: i64,
    pub started_event_id: i64,
    pub timeout_type: TimeoutType,
}

/// `ActivityTaskScheduled` 的属性（`retry_policy` 属 P4，ADR-0013）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityTaskScheduledAttrs {
    pub activity_id: String,
    pub activity_type: String,
    pub task_queue: String,
    pub input: Payload,
    pub schedule_to_close_timeout: Option<Duration>,
    pub schedule_to_start_timeout: Option<Duration>,
    pub start_to_close_timeout: Option<Duration>,
    pub heartbeat_timeout: Option<Duration>,
    pub workflow_task_completed_event_id: i64,
}

/// `ActivityTaskStarted` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityTaskStartedAttrs {
    pub scheduled_event_id: i64,
    pub identity: String,
    pub attempt: i32,
    pub last_failure: Option<Failure>,
}

/// `ActivityTaskCompleted` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityTaskCompletedAttrs {
    pub result: Payload,
    pub scheduled_event_id: i64,
    pub started_event_id: i64,
    pub identity: String,
}

/// `ActivityTaskFailed` 的属性（`retry_state` 属 P4，ADR-0013）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityTaskFailedAttrs {
    pub failure: Failure,
    pub scheduled_event_id: i64,
    pub started_event_id: i64,
    pub identity: String,
}

/// `ActivityTaskTimedOut` 的属性（`retry_state` 属 P4，ADR-0013）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityTaskTimedOutAttrs {
    pub failure: Failure,
    pub scheduled_event_id: i64,
    pub started_event_id: i64,
}

/// `TimerStarted` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimerStartedAttrs {
    pub timer_id: String,
    pub start_to_fire_timeout: Duration,
    pub workflow_task_completed_event_id: i64,
}

/// `TimerFired` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimerFiredAttrs {
    pub timer_id: String,
    pub started_event_id: i64,
}

/// `MarkerRecorded` 的属性；`details` 用 `BTreeMap`（迭代序确定，ADR-0010）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerRecordedAttrs {
    pub marker_name: String,
    pub details: BTreeMap<String, Payload>,
    pub workflow_task_completed_event_id: i64,
    pub failure: Option<Failure>,
}

/// 事件名的官方编号（`temporal/api/enums/v1/event_type.proto`）。
///
/// `as_i16` / `from_i16` 是「事件 ↔ `events.event_type SMALLINT`」的映射；带编号的枚举在单元测试里
/// 与官方表逐项对照，防止 serde 名与数字漂移（ADR-0006）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i16)]
pub enum EventType {
    WorkflowExecutionStarted = 1,
    WorkflowExecutionCompleted = 2,
    WorkflowExecutionFailed = 3,
    WorkflowExecutionTimedOut = 4,
    WorkflowTaskScheduled = 5,
    WorkflowTaskStarted = 6,
    WorkflowTaskCompleted = 7,
    WorkflowTaskTimedOut = 8,
    WorkflowTaskFailed = 9,
    ActivityTaskScheduled = 10,
    ActivityTaskStarted = 11,
    ActivityTaskCompleted = 12,
    ActivityTaskFailed = 13,
    ActivityTaskTimedOut = 14,
    TimerStarted = 17,
    TimerFired = 18,
    MarkerRecorded = 25,
}

impl EventType {
    /// 官方编号。
    pub const fn as_i16(self) -> i16 {
        self as i16
    }

    /// 官方编号 → 事件名；`EVENT_TYPE_UNSPECIFIED`（0）与未实现的事件返回 `None`。
    pub const fn from_i16(value: i16) -> Option<Self> {
        Some(match value {
            1 => Self::WorkflowExecutionStarted,
            2 => Self::WorkflowExecutionCompleted,
            3 => Self::WorkflowExecutionFailed,
            4 => Self::WorkflowExecutionTimedOut,
            5 => Self::WorkflowTaskScheduled,
            6 => Self::WorkflowTaskStarted,
            7 => Self::WorkflowTaskCompleted,
            8 => Self::WorkflowTaskTimedOut,
            9 => Self::WorkflowTaskFailed,
            10 => Self::ActivityTaskScheduled,
            11 => Self::ActivityTaskStarted,
            12 => Self::ActivityTaskCompleted,
            13 => Self::ActivityTaskFailed,
            14 => Self::ActivityTaskTimedOut,
            17 => Self::TimerStarted,
            18 => Self::TimerFired,
            25 => Self::MarkerRecorded,
            _ => return None,
        })
    }
}

/// `WorkflowTaskFailed` 的原因（`temporal/api/enums/v1/failed_cause.proto`）。
///
/// 只留 M1 会真正用到的：非确定性（ADR-0010）、panic（ADR-0007）、以及命令校验失败
/// （ADR-0014 的 `CommandValidationError`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i16)]
pub enum WorkflowTaskFailedCause {
    UnhandledCommand = 1,
    BadScheduleActivityAttributes = 2,
    BadStartTimerAttributes = 4,
    BadRecordMarkerAttributes = 6,
    BadCompleteWorkflowExecutionAttributes = 7,
    BadFailWorkflowExecutionAttributes = 8,
    StartTimerDuplicateId = 12,
    WorkflowWorkerUnhandledFailure = 14,
    ScheduleActivityDuplicateId = 22,
    NonDeterministicError = 24,
}

impl WorkflowTaskFailedCause {
    /// 官方编号。
    pub const fn as_i16(self) -> i16 {
        self as i16
    }
}
