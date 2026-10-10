//! 失败载荷与超时分类（ADR-0013）。

use serde::{Deserialize, Serialize};

/// 失败载荷。
///
/// M1 只需要一个可读的 `message`；P4 按 ADR-0013 把它长成带 `source` / `stack_trace` / `cause` /
/// `failure_info` 四类 variant 的完整 `Failure`。现在不预置用不上的字段。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Failure {
    pub message: String,
}

/// 超时分类；落库时用 Temporal 官方 `TimeoutType` 编号（`temporal/api/enums/v1/workflow.proto`）。
///
/// M1 只有 WFT 超时（`StartToClose`）；P4 补齐 activity 的四种超时判定（ADR-0013）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i16)]
pub enum TimeoutType {
    StartToClose = 1,
    ScheduleToStart = 2,
    ScheduleToClose = 3,
    Heartbeat = 4,
}

impl TimeoutType {
    /// 官方编号。
    pub const fn as_i16(self) -> i16 {
        self as i16
    }

    /// 官方编号 → 分类；`UNSPECIFIED`（0）与未实现的值返回 `None`。
    pub const fn from_i16(value: i16) -> Option<Self> {
        Some(match value {
            1 => Self::StartToClose,
            2 => Self::ScheduleToStart,
            3 => Self::ScheduleToClose,
            4 => Self::Heartbeat,
            _ => return None,
        })
    }
}
