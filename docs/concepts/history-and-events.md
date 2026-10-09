# History 与 Events

## Temporal 怎么做

历史是一个 execution 的**唯一真相**，由 60 余种事件类型构成。`HistoryEvent` 的 envelope 是 `event_id`（从 1 起单调）、`event_time`、`event_type`、`version`、`task_id`、`worker_may_ignore`、`user_metadata`、`links`、`principal` 加一个 `oneof attributes`。

三条不变量：事件 id 单调；每个 run **恰好一个**闭合事件（`Completed`/`Failed`/`Canceled`/`Terminated`/`ContinuedAsNew`/`TimedOut`）且它是最后一条；闭合命令之后不再有命令。

关联靠两种回指：命令作用在**已存在**的事件上时带 `scheduled_event_id`（例如取消活动）；命令派生的**新**事件带 `workflow_task_completed_event_id`，指回是哪个 WFT 完成了它。非确定性的比对**由 SDK 在重放时做**，不是服务端拿新旧命令对账。

存储上，事件写进 `history_node`/`history_tree` 这棵分页树；另有一批事件属于 **buffered / transient**（生命周期事件、全部 WFT 事件、命令派生事件、消息派生事件之外的都默认进 buffer），未提交前不落历史。

## 我们怎么做

- 一张平表：`events(run_id, event_id, event_type, event_time, attributes)`，`event_type` 用 Temporal 官方编号，`attributes` 是 JSONB（ADR-0006）。
- 回指字段（`workflow_task_completed_event_id` / `scheduled_event_id`）照样写 —— 于是 SQL 里能直接 join 出「哪个 WFT 产生了哪些事件」。
- 事件 id 由 `executions.next_event_id` 分配，与状态更新在同一个 OCC 事务里。
- 每次重放都读**全部**事件（ADR-0002）。

## 代价

D4：无分页树，超长历史全量读取代价高。D5：无 transient/buffered，重试的每次尝试都留在历史里（换来「至少一次」的可见性）。
