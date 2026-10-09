# Versioning 与打补丁

## Temporal 怎么做

长跑的 execution 会跨越多次代码部署，而重放要求「同样的历史 ⇒ 同样的调用序列」。于是改代码必须留下**可重放的分叉标记**：

```go
if workflow.GetVersion(ctx, "add-shipping", DefaultVersion, 2) == 2 { … }
```

`GetVersion(change_id, min, max)` 的值落进 `MarkerRecorded` 事件；重放旧历史时返回旧值，新历史返回新值。`patched` 是它的单版本简化（返回 bool）。**纪律是两步的**：先加 `patched`，等带旧标记的历史全部跑完之后，再把它替换成 `true` 并删除标记 —— 过早删除会让仍在重放的旧 execution 报非确定性。

另一条路是 **worker versioning**（`Pinned` / `AutoUpgrade` + Build ID），让 execution 钉在某个 worker 版本上，从根本上不需要 patch —— 那是运维向的机制。

## 我们怎么做

P5 实现 `ctx.patched(change_id) -> bool`：内部就是一个 `MarkerRecorded`（`marker_name = "Version"`），复用 M1 的 marker 机制，成本近乎为零（ADR-0015 把它从 P6 提前）。`oxidura replay-check <run_id>` 是配套的验证工具：改完代码拿真实历史重放一遍，看是否报 `NonDeterminism`。

## 代价

D11：没有 `GetVersion` 的区间语义（同时存在 3 个以上版本时才会用到）。D10：不做 worker versioning，所以「钉版本」这条替代路径只能从文档里了解。
