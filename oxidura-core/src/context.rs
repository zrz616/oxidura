//! `WorkflowContext` 与 workflow 作者的错误类型（ADR-0009）。
//!
//! M1-02 只放骨架：`replay` 在内部创建 ctx 并交给 `entry`，业务代码还拿不到任何能力。

/// workflow 作者返回的错误。
///
/// ADR-0009 定名 `WfError`（不是 `WorkflowError`）。P4 按 ADR-0013 把它长成能包住
/// `ActivityFailure` 的完整错误；M1 只留一个 message。
#[derive(Debug, Clone, PartialEq)]
pub struct WfError {
    pub message: String,
}

/// 工作流代码手里的句柄；将来是 `Rc<RefCell<Inner>>` 的薄壳，按值传（ADR-0003）。
///
/// M1-02 只是占位：`execute_activity` / `sleep` / `now` / … 在后续切片长出来。
/// 字段私有 —— 只有 `replay` 能造，调用方拿不到旧 ctx（这也让 ADR-0002 的「每轮丢弃内存态」
/// 变成结构性保证）。
#[derive(Debug)]
pub struct WorkflowContext {
    _private: (),
}

impl WorkflowContext {
    /// `replay` 每轮新建一个。字段保持私有 —— 调用方永远拿不到上一轮的 ctx，
    /// ADR-0002 的「每轮丢弃内存态」因此是结构性保证。
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}
