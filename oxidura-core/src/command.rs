//! 命令形状：`Command` oneof 与命令的官方数字编号（ADR-0005、ADR-0006）。

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::Payload;
use crate::failure::Failure;

/// 一轮重放产出的命令；serde 的内部 tag **就是 Temporal 的命令名**（ADR-0006）。
///
/// M1 命令子集（ADR-0005）：`ScheduleActivityTask` / `StartTimer` / `RecordMarker` /
/// `CompleteWorkflowExecution` / `FailWorkflowExecution`。每个 variant 用 `Box` 包住属性结构体。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Command {
    ScheduleActivityTask(Box<ScheduleActivityTaskAttrs>),
    StartTimer(Box<StartTimerAttrs>),
    CompleteWorkflowExecution(Box<CompleteWorkflowExecutionAttrs>),
    FailWorkflowExecution(Box<FailWorkflowExecutionAttrs>),
    RecordMarker(Box<RecordMarkerAttrs>),
}

impl Command {
    /// 命令名对应的官方编号。
    pub fn command_type(&self) -> CommandType {
        match self {
            Self::ScheduleActivityTask(_) => CommandType::ScheduleActivityTask,
            Self::StartTimer(_) => CommandType::StartTimer,
            Self::CompleteWorkflowExecution(_) => CommandType::CompleteWorkflowExecution,
            Self::FailWorkflowExecution(_) => CommandType::FailWorkflowExecution,
            Self::RecordMarker(_) => CommandType::RecordMarker,
        }
    }
}

/// `ScheduleActivityTask` 的属性（`retry_policy` 属 P4，ADR-0013）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleActivityTaskAttrs {
    pub activity_id: String,
    pub activity_type: String,
    pub task_queue: String,
    pub input: Payload,
    pub schedule_to_close_timeout: Option<Duration>,
    pub schedule_to_start_timeout: Option<Duration>,
    pub start_to_close_timeout: Option<Duration>,
    pub heartbeat_timeout: Option<Duration>,
}

/// `StartTimer` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StartTimerAttrs {
    pub timer_id: String,
    pub start_to_fire_timeout: Duration,
}

/// `CompleteWorkflowExecution` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompleteWorkflowExecutionAttrs {
    pub result: Payload,
}

/// `FailWorkflowExecution` 的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailWorkflowExecutionAttrs {
    pub failure: Failure,
}

/// `RecordMarker` 的属性；`details` 用 `BTreeMap`（迭代序确定，ADR-0010）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordMarkerAttrs {
    pub marker_name: String,
    pub details: BTreeMap<String, Payload>,
    pub failure: Option<Failure>,
}

/// 命令名的官方编号（`temporal/api/enums/v1/command_type.proto`）。
///
/// 编号与「命令 → 事件」的映射（ADR-0008）一起，是提交路径对账的基础。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i16)]
pub enum CommandType {
    ScheduleActivityTask = 1,
    StartTimer = 3,
    CompleteWorkflowExecution = 4,
    FailWorkflowExecution = 5,
    RecordMarker = 9,
}

impl CommandType {
    /// 官方编号。
    pub const fn as_i16(self) -> i16 {
        self as i16
    }

    /// 官方编号 → 命令名；`COMMAND_TYPE_UNSPECIFIED`（0）与未实现的命令返回 `None`。
    pub const fn from_i16(value: i16) -> Option<Self> {
        Some(match value {
            1 => Self::ScheduleActivityTask,
            3 => Self::StartTimer,
            4 => Self::CompleteWorkflowExecution,
            5 => Self::FailWorkflowExecution,
            9 => Self::RecordMarker,
            _ => return None,
        })
    }
}
