//! 重放内核：事件、命令、确定性时钟，以及 `replay()`。
//!
//! 这一层不认识 SQL，也不跑在 async runtime 上——那是 crate 边界要保证的事（ADR-0003、ADR-0004）。
//!
//! M1-01（TOM-10）先给出重放内核的线格式：三组形状 `HistoryEvent` / `Event` / `Command`，
//! 它们对应的官方数字编号（将来是 `events.event_type SMALLINT` / `executions.status SMALLINT`），
//! 以及纯函数 [`apply_commands`]（`Command` → `HistoryEvent` 的物化）。重放引擎本身在 M1 的后续
//! 切片里长出来（`docs/milestones/m1.md`）。
//!
//! # 依赖
//!
//! `serde` / `serde_json` / `chrono` 只服务「类型形状与序列化」（ADR-0006）。`core` **不认识**
//! 数据库与 async runtime（`sqlx` / `tokio`），这条约束由 gate 检查。

pub mod command;
pub mod event;
pub mod failure;
pub mod status;

mod apply;

pub use apply::apply_commands;
pub use command::{Command, CommandType};
pub use event::{Event, EventType, HistoryEvent, WorkflowTaskFailedCause};
pub use failure::{Failure, TimeoutType};
pub use status::WorkflowExecutionStatus;

/// workflow 输入 / activity 输入 / 任意结果。
///
/// Temporal 的 `Payloads` 在这里塌缩成单个 JSON 值（D2、ADR-0006）：没有 codec、没有加密、没有
/// 非 JSON 编码。将来需要时加 `PayloadConverter` 的实现，不动这个别名。
pub type Payload = serde_json::Value;
