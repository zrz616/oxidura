# Timers

## Temporal 怎么做

命令是 `StartTimer(timer_id, start_to_fire_timeout)`，产生 `TimerStarted` 事件；`CancelTimer(timer_id)` 与它配对。timer 是**持久化**且可靠的：等待期间不占用任何 worker 资源，时长是**最小值**（触发时间向上取整）。

服务端两侧存储：durable 状态在 `timer_info_maps`，触发计划在 `timer_tasks`（主键含 `visibility_timestamp`）。到期由 **History 分片的 timer 队列处理器**接管：追加 `TimerFired` + `WorkflowTaskScheduled`，再由 transfer 队列把 WFT 交给 Matching。WFT 启动时服务端还会为 WFT 超时建一个 timer task；activity 启动时同样为 activity 超时建 timer task。

## 我们怎么做

- `ctx.sleep(d)` → `StartTimer` 命令 → `TimerStarted` 事件 + 一行 `tasks(task_type = TIMER, visible_at = TimerStarted.event_time + d)`。
- 到期由 scanner 处理：写 `TimerFired` + `WorkflowTaskScheduled` + 排新 WFT（ADR-0012）。
- `timer_id` 由 `seq` 派生（ADR-0014）。注意**同一个 sleep 在重放中必须得到同一个 id**，这正是 seq 派生要解决的问题。
- M1 阶段 `ctx.sleep` 已在内存 `TestWorkflowEnvironment` 里可用，由虚拟时钟 `advance_time(d)` 驱动；**持久化**的 timer 任务从 P4 开始。

## 代价

D6：timer 与其它任务共用一张表，因此没有 Temporal 那种「History 的 timer 队列」这一层 —— 好处是语义链条更短，坏处是少了一处「三类队列如何协作」的观察点。
