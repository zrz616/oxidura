# ADR-0003 同步 poll-to-quiescence 驱动重放

**决定**：工作流代码**不跑在任何 async runtime 上**。`replay()` 用 noop waker 手动 poll root future 直到 Pending，单轮重放是一个纯同步推进过程。由此：

- `WorkflowContext` 内部是 `Rc<RefCell<Inner>>`，按值传、廉价 clone，`!Send`。
- 每次 poll 有 `PollBudget` 上限，超限返回 `PollBudgetExceeded`。
- **硬约束**：单次 replay 内不得出现「依赖 waker 被触发才能推进」的 future。每个 await 只有两种结局 —— 历史命中立刻 Ready，或无历史则 Pending 并登记命令。

**为什么**：工作流代码永不真正等待 I/O（活动结果与时间都来自历史），所以 runtime 是多余的。`!Send` 让「不得把工作流丢到别的线程」从纪律变成编译错误；`Rc<RefCell>` 把「跨 await 持有借用」从静默死锁变成立刻 panic（`BorrowMutError`）—— 学习项目里，响亮失败远胜安静挂起。`Arc<Mutex>` 只在需要 future `Send` 时才值得，而我们不需要。

**代价**：不能用 `tokio::spawn` 跑重放（也不需要）；`PollBudget` 是必须的，否则一个自己唤醒自己的 future 会把 worker 卡死在同步循环里。

**相关**：ADR-0002 · ADR-0009
