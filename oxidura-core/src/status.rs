//! `executions.status` 的取值（ADR-0006、ADR-0011）。

use serde::{Deserialize, Serialize};

/// 一次 workflow execution 的状态；落库为 `executions.status SMALLINT`。
///
/// 用 Temporal 官方 `WorkflowExecutionStatus` 编号：SQL 里的数字能直接与
/// `temporal/api/enums/v1/workflow.proto` 对照，`RUNNING = 1` 正是 Temporal 在 schema 里
/// hardcode 的值（ADR-0006）。`Paused` 等本仓库「永不做」的状态不在这里留列（§2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i16)]
pub enum WorkflowExecutionStatus {
    Running = 1,
    Completed = 2,
    Failed = 3,
    Canceled = 4,
    Terminated = 5,
    ContinuedAsNew = 6,
    TimedOut = 7,
}

impl WorkflowExecutionStatus {
    /// 官方编号。
    pub const fn as_i16(self) -> i16 {
        self as i16
    }

    /// 官方编号 → 状态；`UNSPECIFIED`（0）与未实现的值返回 `None`。
    pub const fn from_i16(value: i16) -> Option<Self> {
        Some(match value {
            1 => Self::Running,
            2 => Self::Completed,
            3 => Self::Failed,
            4 => Self::Canceled,
            5 => Self::Terminated,
            6 => Self::ContinuedAsNew,
            7 => Self::TimedOut,
            _ => return None,
        })
    }
}
