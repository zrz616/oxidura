# ADR-0004 工作区分层与进程拓扑

**决定**：六个 crate，依赖单向向内：

`oxidura-core`（无 tokio / 无 sqlx）← `oxidura-store-postgres`、`oxidura-sdk(-macros)` ← `oxidura-worker`（bin `oxidura`）。

进程拓扑：`oxidura worker --role workflow|activity|both`，**默认起两个独立进程**。

**为什么**：crate 边界就是依赖边界。「core 不认识 SQL」这条约束是**让 M1「不连数据库」可验证的机制**，不是风格偏好 —— 单 crate 下这条只能靠自觉，几周后必然被 `use sqlx` 污染，所以 M0 的 gate 直接用 grep 强制它。`oxidura-sdk-macros` 独立是因为 `proc-macro = true` 的 crate 不能同时导出运行时代码。

分进程的理由是**验收的真实性**：M3 要验证「worker 被 `kill -9` 后能恢复」。单进程里一杀全死，测试会退化成「重启整个东西」，恰好掩盖掉「workflow task 幂等与重入」这个真正的难点。

**代价**：六 crate 的样板开销；两进程需要各自的 CLI 与日志。

**相关**：ADR-0015 · `DESIGN.md` §3
