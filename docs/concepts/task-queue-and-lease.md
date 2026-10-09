# Task Queue 与租约

## Temporal 怎么做

三类 task queue（Activity / Workflow / Nexus）按需创建，一个队列名自动派生三类子队列。worker 用同步长轮询取任务；默认 4 个分区，单分区时近似 FIFO。

**Temporal 的核心队列不用 `FOR UPDATE SKIP LOCKED`**：它靠 `range_id` 世代号 fencing（`SELECT range_id FROM task_queues_v2 … FOR UPDATE`，写入带 `range_id` 条件，失配即 `ConditionFailedError`、整批被拒）+ 应用层「同一时刻只有一个 writer」的串行化 + 事务性发件箱把 History 与 Matching 串起来。待处理任务是普通 `SELECT … ORDER BY task_id LIMIT`。

Temporal 里没有「租约」这个词：任务的超时由三类 timeout 表达（详细见 `timers.md`），worker 死后靠 start-to-close / schedule-to-start 超时把任务放回去。

## 我们怎么做

- 一张 `tasks` 表四种 `task_type`；`SKIP LOCKED` + **单语句** `UPDATE … WHERE task_id IN (SELECT … FOR UPDATE SKIP LOCKED) RETURNING *` 完成「选中即占住」（ADR-0011）。
- 显式租约：60s，每 20s 续租，归属校验用 `worker_identity`。
- 完成后 `DELETE` —— 任务行是投递状态，事实在 `events`。
- 过期只能由 **scanner** 放开（ADR-0012）；`dequeue` 永不承认过期租约。
- `dequeue(queues: &[&str], …)` 接多队列，为 P6 的 sticky 预留。

## 代价

无 fencing：worker 假死时任务滞留到租约过期为止。`range_id` 方案留作 P6 对照实验（那时你会有两套实现可对比行为差异）。
