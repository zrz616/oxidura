# Workflow Task

## Temporal 怎么做

WFT 是「推进一个 execution 一步」的调度单元。服务端在这些时刻排一个新的 WFT：execution 启动、收到 signal/update、activity 结束、timer 到期、子 workflow 结束、以及 WFT 自身失败需要重试。每个 WFT 把「自上次 WFT 以来新发生的事件」打包在一起。

一个逻辑 WFT 在历史里是：`[事件][命令]* ; WorkflowTaskScheduled ; WorkflowTaskStarted ; WorkflowTaskCompleted`。`WorkflowTaskStarted` 在**派发时、worker 跑代码之前**写入，而且这个 RPC 就把历史返回给 worker —— **边界包含本轮的 `Started`**。命令与由此产生的**事件在同一事务**里追加。

超时默认 10s（上限 120s）。WFT 失败（非确定性、未捕获异常、命令非法、超时）**不会**让 execution 失败：run 保持 Open，按指数退避重试，上限 10 分钟、默认直到 `workflow_execution_timeout`（默认无限）。WFT 不受 RetryPolicy 约束。完成后**不会**自动再排一个 WFT，除非 `force_create_new_workflow_task` 等少数条件成立。

## 我们怎么做

- 事件五件齐全（`Scheduled`/`Started`/`Completed`/`Failed`/`TimedOut`）。
- 写入时机按 ADR-0008：`Scheduled` 与触发事件同事务；worker 拿到租约后先写 `Started`，再读 replay 边界；`Completed` 与物化事件同事务。
- 超时用**租约**表达：worker 领任务即 LEASED，过期由 scanner 写 `WorkflowTaskTimedOut` 并重排（ADR-0012）。
- 失败语义按 ADR-0007：panic / 非确定性 / 命令校验失败 → `WorkflowTaskFailed`，run 保持 Open 并退避重试；退避上限是引擎配置（开发期 10s）。

## 代价

D5：Temporal 对第 2 次及以后的 WFT 尝试用 transient 事件（不写历史），我们每次都写 —— 好处是「重试了几次、每次长什么样」在历史里直接可见。
