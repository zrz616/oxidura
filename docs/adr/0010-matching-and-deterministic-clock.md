# ADR-0010 匹配算法、跳过白名单与确定性时钟

**决定**：

- **严格顺序 + 期望类型匹配**：`ctx` 维护游标，每个 await 点消费事件并要求类型（及 `activity_id` / `timer_id`）相符；不符即返回 `NonDeterminism { at_event_id, expected, actual }`。
- **跳过白名单做成谓词** `is_command_correlated(event_type) -> bool`：`WorkflowTaskScheduled` / `Started` / `Completed` / `Failed` / `TimedOut`、`ActivityTaskStarted` 属于「引擎产物」，被跳过而不消费。
- **AsyncEvent 分流**：`WorkflowExecutionSignaled` 这类**非需求驱动**的事件不由游标消费，游标经过时投入 ctx 的按名缓冲（D13）。
- **确定性时钟**：`ctx.now()` = replay 边界里最后一条 `WorkflowTaskStarted.event_time`。`ctx.random()` / `uuid()` / `side_effect()` 的值落进 `MarkerRecorded`。
- 禁止清单（写进 `concepts/determinism.md`，不设机制）：`HashMap` 迭代序、`std::time::Instant`/`SystemTime`、`rayon`、任何真实 I/O。

**为什么**：位置性匹配让漂移**在第一次发生时就定位到具体 `event_id`**，而按 id 查找（拿 `timer_id` 去历史里 find）会把漂移推迟到「最后对不上账」时才暴露。`ctx.now()` 从历史取，重放与 live 天然一致 —— 这反过来证明 `WorkflowTaskStarted` 必须落在 replay 边界之内（ADR-0008 的 `WorkflowTaskStarted` 写入时机）。

**代价**：D13 —— signal 的消费是**非位置性**的，因此它天然比命令对应事件缺少非确定性的保护（改变等待顺序不会被立刻发现）。这是 Temporal 语义的真实性质，不是本实现的缺陷。

**相关**：ADR-0002 · ADR-0005 · ADR-0014
