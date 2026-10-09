# Sticky Cache 与 Sticky Queue

## Temporal 怎么做

worker 内部维护一个 LRU 形式的 **workflow cache**：重放过的 execution 的内存态留在进程里。**Sticky execution** 让后续 WFT 优先投给缓存过它的那个 worker，方法是一个每实例唯一命名的 **sticky queue**（名字随重启变化）。

- worker 同时轮询共享队列与自己的 sticky 队列。
- sticky WFT 若在 **schedule-to-start 超时（默认 5s）** 内没被领走，服务端关闭 stickiness，把任务放回原队列。
- WFT 失败时 worker 驱逐该 execution 的缓存（状态已不可信），stickiness 随之失效。
- sticky poller 可以只拿**部分**历史：从 `previousStartedEventId + 1` 开始。

这是 Temporal 纯粹为了**性能**存在的一层：正确性由「从零重放」保证，缓存只是把重复的那部分省掉。

## 我们怎么做

P6 才做，并且是作为**对照实验**（ADR-0016）：

- worker 内进程 LRU + 部分重放：只读 `event_id > 最后一条 WorkflowTaskStarted 的 event_id`。
- sticky 队列名 `sticky-<uuid>`，5s 未领取由 scanner 回落到共享队列（复用 ADR-0012，不新增代码路径）。
- **无 schema 变更**：「进度」本来就在历史里，内存态由 worker 自己持有。
- `--bench` 在一条 10,000 事件的历史上量三个数：每轮重放的事件数、重放耗时、从 PG 读的行数与字节数。

## 代价

D12：缓存不跨进程共享，进程退出即丢，恢复只能靠全量重放。这正是「sticky 是优化而非正确性机制」的证明。
