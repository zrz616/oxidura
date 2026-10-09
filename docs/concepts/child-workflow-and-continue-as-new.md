# Child Workflow 与 Continue-As-New

## Temporal 怎么做

**子 workflow** 是完整的另一个 execution（有自己的 `run_id` 与历史）。父方发 `StartChildWorkflowExecution` 命令 → `StartChildWorkflowExecutionInitiated` 事件 → 服务端起子 execution → `ChildWorkflowExecutionStarted` / `Completed` / `Failed` / `TimedOut` / `Canceled` 回报父方。父关闭时子怎么处理由 `ParentClosePolicy` 决定，**默认 `TERMINATE`**（另两个是 `ABANDON`、`REQUEST_CANCEL`）。子启动同样受 `WorkflowIdReusePolicy` 约束，**默认 `ALLOW_DUPLICATE`**。

**Continue-As-New（CAN）** 解决「历史无限增长」：workflow 发 `ContinueAsNewWorkflowExecution`，当前 run 以 `WorkflowExecutionContinuedAsNew` 关闭，同时以**同一个 `workflow_id`**、**新的 `run_id`** 起新 run；新 run 的历史从零开始。服务端会在历史过大时通过 `suggest_continue_as_new` 提示，原因是 `HISTORY_SIZE_TOO_LARGE` / `TOO_MANY_HISTORY_EVENTS` / `TOO_MANY_UPDATES`。注意 `workflow_execution_timeout` 跨整条 CAN 链，所以 CAN 命令**不允许**覆盖它。

## 我们怎么做

- P5 实现 `ctx.child_workflow(name, input, opts).await`，父关闭策略只做 `TERMINATE`（默认值）；子跑在同一 task queue。
- P5 实现 `ctx.continue_as_new(input)` + 历史长度阈值：超阈值时在 WFT 完成响应里带建议标记，但**不自动执行** —— 是否 CAN 是业务决定。
- 身份模型：`executions` 一行一个 run，`workflow_id` 是业务键，`(workflow_id) WHERE status='RUNNING'` 唯一（ADR-0011 的不变量），于是「同一个 workflow_id 起第二次会得到新 run」这件事由 schema 天然表达。
- 客户端 API 侧区分两个枚举：`WorkflowIdReusePolicy` 管**已关闭**的 run（默认 `ALLOW_DUPLICATE`），`WorkflowIdConflictPolicy` 管**正在跑**的（M3 只做 `FAIL`，其余留 P5）。

## 代价

P5 之前只能手写「父 workflow 里串起两段逻辑」，因此「两个 execution 的独立性」这一课要等到 P5。
