# ADR-0008 Command → Event 物化与提交路径

**决定**：物化是**引擎的职责**，且是一个 core 里的纯函数：

```rust
fn apply_commands(history_len: i64, wft_completed_event_id: i64,
                  now: DateTime<Utc>, cmds: &[Command]) -> Vec<HistoryEvent>
```

返回 `HistoryEvent`（envelope）而非裸 `Event`：`history_len` 是 `event_id` 的起点（产出事件从
`history_len + 1` 起连续取号），`now` 是它们的 `event_time` —— 两者只有在返回 envelope 时才有意义
（ADR-0006）。`scheduled_event_id` 型回指（命令作用在**既有**事件上）随 P4 的 cancel 命令引入；M1 的
命令子集产出的都是「派生新事件」，只带 `workflow_task_completed_event_id` 回指。

worker 完成一次重放后，在**一个事务**里：物化事件 + 分配 `event_id`（`next_event_id`）+ 回填 `workflow_task_completed_event_id` / `scheduled_event_id` 回指 + enqueue 后续任务。OCC 用一条语句完成 CAS：

```sql
UPDATE executions SET next_event_id = …, db_record_version = db_record_version + 1
WHERE run_id = $1 AND db_record_version = $2 RETURNING next_event_id
```

外部世界注入的事件（`TimerFired`、`ActivityTaskCompleted`、`WorkflowExecutionSignaled`）**不走**这个函数。

**为什么**：对应 Temporal 的「一步状态转移 = 追加事件 + 更新可变状态 + 加队列任务，全部同事务」。把它做成纯函数的好处是**M1 就能测它**（给定命令 → 断言事件序列与回指字段），完全不需要数据库 —— 这是 M1「不连数据库也能验证内核」的支点。

注意 D7：Temporal 的事务性发件箱是为了**跨服务**原子性（History 与 Matching 是两个进程）。我们只有一个 PostgreSQL，同事务 enqueue 就是等价且更简单的正确解 —— 学 Temporal 不等于抄它的架构。

**代价**：D7；`apply_commands` 的签名不得假设「必有闭合命令」（P5 的 signal 醒来后继续跑的场景没有闭合命令）。

**相关**：ADR-0011 · ADR-0012 · `concepts/history-and-events.md`
