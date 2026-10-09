# ADR-0016 P6 基准与实验集设计

**决定**：P6 交付一份**可复现基准报告**，不是「实现了 sticky」：

- 固定基准：一条 **N = 10,000 事件**的历史。
- 度量三项：① 每轮 WFT 重放的事件数；② 单轮重放耗时；③ 从 PG 读取的行数 / 字节数。
- sticky 实现：worker 内进程 LRU + 部分重放（只读 `event_id > 最后一条 WorkflowTaskStarted 的 event_id`）+ `sticky-<uuid>` 队列（5s 未领取则由 scanner 回落到共享队列）—— 复用 ADR-0011 的多队列签名与 ADR-0012 的 scanner。
- **无 schema 变更**：缓存进度就是「最后一条 `WorkflowTaskStarted` 的 `event_id`」，它本来就在历史里，而内存态由 worker 自己持有。
- 实验集：`range_id` fencing 对照、transient / buffered events 对照。
- 交付物：`--bench` 子命令 + `docs/milestones/p6.md` 里的结论段落。

**为什么**：先有正确性基线（M1–P5），量化才有意义。P6 同时是「为什么 Temporal 要花大力气做 sticky queue 与缓存」这一问题的**唯一诚实答案**——你可以亲手复现差距，而不是背结论。

**代价**：D12（缓存不跨进程共享，进程退出即丢，只能靠全量重放恢复）。

**相关**：ADR-0002 · ADR-0011 · ADR-0012 · `concepts/sticky-cache.md`
