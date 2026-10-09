# ADR-0005 事件与命令：镜像 Temporal 命名，只实现子集

**决定**：类型名与语义**精确镜像 Temporal**（`WorkflowTaskScheduled`、`ActivityTaskScheduled`、`TimerStarted`…），字段只留需要的；envelope 精简为 `event_id` / `event_time` / `attributes`（D1）。

M1 事件子集：`WorkflowExecutionStarted`、WFT 全五件（`Scheduled`/`Started`/`Completed`/`Failed`/`TimedOut`）、`ActivityTaskScheduled`/`Started`/`Completed`/`Failed`/`TimedOut`、`TimerStarted`/`TimerFired`、`MarkerRecorded`、`WorkflowExecutionCompleted`/`Failed`/`TimedOut`。

M1 命令子集（沿用 Temporal 官方编号）：`ScheduleActivityTask`、`StartTimer`、`RecordMarker`、`CompleteWorkflowExecution`、`FailWorkflowExecution`。

P4 补 `CancelTimer`/`TimerCanceled`；P5 补 signal/child/CAN/cancel 相关。

**为什么**：命名即概念。照抄名字之后，读 Temporal 文档或 sdk-core 源码时的映射成本为零，而自创一套 Rust 味命名（`Event::WfStarted`）会把这份映射成本永久留在项目里。

**WFT 三件套不可压缩**：Temporal 与「直接调函数」的唯一分水岭，就是「WFT 是一个可被独立调度、独立失败、独立超时的单元」。压成一个事件后，WFT 超时、WFT 失败重试、transient WFT 全部无处安放。

**代价**：D1（无 worker versioning 相关排查能力）。

**相关**：ADR-0006 · ADR-0007 · `concepts/history-and-events.md`
