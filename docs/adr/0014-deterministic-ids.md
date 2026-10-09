# ADR-0014 确定性 ID 生成

**决定**：`ctx` 维护一个**每轮重放内自 0 递增**的 `seq`（不入库）。`timer_id` 与 `activity_id` 默认由它派生（`format!("{seq}")`）；用户可显式覆盖 `activity_id`（例如用业务键 `charge-{order_id}`）。引擎校验「同一 execution 内不得有两个存活的同名 id」，违反则让 WFT 失败并携带 `CommandValidationError`。

**为什么**：Temporal 的 proto 对 `activity_id` 没有任何约束性注释，对 `timer_id` 只要求 `currently live timers must have different ids` + 通常由 SDK 自动生成。两份约束我们都满足，而 `seq` 派生还额外白送一条性质：**不需要持久化就能确定**。若把 id 存进 DB 才能复现，就等于承认「重放位置不足以决定 ID」——那是设计错误。附带收益：`CommandValidationError` 把「命令属性校验」变成显式枚举，之后所有调试体验都建立在它上面。

**代价**：与 Temporal 各 SDK 各自的自动生成算法未必一致（未核实，且不影响互操作，因为我们不与 Temporal 通信）。

**相关**：ADR-0010 · ADR-0005
