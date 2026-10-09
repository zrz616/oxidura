//! 示例 workflow：订单履约（`charge_card` → `ship_order` → `notify_customer`）。
//!
//! 这个示例随里程碑生长：M3 三个 activity，P4 插入 `ctx.sleep`，P5 插入 `wait_signal("cancel")`，
//! P6 加补偿。各阶段的形态见对应的 `docs/milestones/*.md`。
