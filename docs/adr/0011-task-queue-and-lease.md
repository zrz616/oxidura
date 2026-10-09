# ADR-0011 任务队列与租约

**决定**：一张 `tasks` 表承载四种任务类型（`WORKFLOW` / `ACTIVITY` / `TIMER` / `QUERY`），列含 `task_queue`、`run_id`、`scheduled_event_id`、`activity_id`、`attempt`、`visible_at`、`state`、`lease_expires_at`、`worker_identity`。

- 取任务用**单语句**：`UPDATE tasks SET state='LEASED', lease_expires_at=… WHERE task_id IN (SELECT … FOR UPDATE SKIP LOCKED) RETURNING *` —— 一次往返、无竞态窗口。
- 租约默认 60s，worker 每 20s 续租，续租与完成都带 `WHERE worker_identity = $me` 的归属校验。
- 完成即 `DELETE`：任务行是**投递状态**，不是事实；事实在 `events` 表。
- `dequeue(queues: &[&str], …)` **接受多个队列** —— 这是全项目唯一一处为 P6 sticky 预留的签名级形状。

**为什么**：`SKIP LOCKED` 让多个 worker 实例天然安全；单语句 `UPDATE … RETURNING` 把「选中」与「占住」合成原子操作，避免两次往返之间的竞态。多队列签名是为了 P6 —— sticky 需要 worker 同时轮询共享队列与自己的 `sticky-<uuid>` 队列，而改这个签名会波及 worker 循环与全部测试。

**代价**：没有 range_id fencing（Temporal 的做法）。worker 假死时任务滞留到租约过期为止，由 scanner 兜底（ADR-0012）；range_id 方案留作 P6 对照实验。

**相关**：ADR-0012 · ADR-0016 · `concepts/task-queue-and-lease.md`
