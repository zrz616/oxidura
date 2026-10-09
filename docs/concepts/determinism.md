# Determinism

## Temporal 怎么做

规则一句话：**同样的输入，工作流代码必须做出同样的 API 调用序列**。因此这些都要换掉：

| 关切 | 禁止 | 替代 |
|---|---|---|
| 墙上时钟 | `time.Now()` / `Date.now()` | `workflow.Now(ctx)`，值 = 当前 WFT 的 `WorkflowTaskStarted.event_time` |
| 随机 / UUID | 裸 RNG | SideEffect，或 `workflow.random()` / `uuid4()` |
| I/O | workflow 里直接调 | 放进 activity |
| 协程 / 线程 | `go` / `chan` / `Thread` | `workflow.Go` / `Channel` / `Selector` |
| map 迭代 | 顺序随机 | 先排序键 |
| mutating 全局 | — | 状态放实例内 |

非确定性由 **SDK 在重放时**发现，报成 WFT 失败（`NON_DETERMINISTIC_ERROR`），run 保持 Open 并重试 —— 这就是「发版引入非确定性 → execution 全部卡在重试 → 回滚后自动续跑」这一真实运维故事的机制。

## 我们怎么做

- 所有非确定性来源都从 `ctx` 走：`now()` / `random()` / `uuid()` / `side_effect()`，它们的值都落进历史（后三者是 `MarkerRecorded`）。
- 匹配器按 ADR-0010 在**第一次不匹配时**报 `NonDeterminism { at_event_id, expected, actual }`。
- 禁止清单（`HashMap` 迭代序、`Instant`/`SystemTime`、`rayon`、真实 I/O）写在这里，靠 review 与 `replay-check` 守，不设机制。
- 安全改代码靠 `ctx.patched(change_id)`（见 `versioning.md`）。

## 代价

D13：signal 的消费不是位置性的，所以「改变等待顺序」这类漂移**不会**被立刻发现 —— 这是 Temporal 语义的真实性质，不是本实现的漏洞。
