//! 重放内核：事件、命令、确定性时钟，以及 `replay()`。
//!
//! 这一层不认识 SQL，也不跑在 async runtime 上——那是 crate 边界要保证的事（ADR-0003、ADR-0004）。
//!
//! M0 只有骨架；内核本身在 M1 长出来（`docs/milestones/m1.md`）。
