# ADR-0007 三类失败出口与超时

**决定**：所有失败归入三档，各自有不同的可见后果：

| 出口 | 触发 | 历史 | run 状态 |
|---|---|---|---|
| **execution failure** | workflow 返回 `Err(WfError)`，或显式 `FailWorkflowExecution` | `WorkflowExecutionFailed` | 关闭（终态） |
| **workflow-task failure** | 重放 panic、非确定性、命令校验失败、WFT 超时 | `WorkflowTaskFailed` / `WorkflowTaskTimedOut` | **保持 Open**，按退避重试 |
| **lease failure** | DB 断连、`kill -9`、worker 假死 | 由 scanner 随后补写 `*TimedOut` | 不变，任务重投 |

配套：`catch_unwind` 包裹**每一轮重放**与**每一次 activity 执行**（panic 是 WFT 失败/活动失败，不是进程崩溃）；实现 `workflow_execution_timeout`（默认无限，为 `0`）；`wft_retry_backoff_cap` 作引擎配置，开发期设 10s。

**为什么**：这是 Temporal 的原始规则 —— 只有 Temporal Failure 才失败 execution，其它一律失败 WFT 并重试（Go 侧原话：返回 error 失败 execution，panic 失败 WFT）。Rust 里没有「未捕获异常」，映射关系反而更干净：**`Err` = execution failure，panic = workflow-task failure**。同时，Temporal 默认「无限重试 + 无限超时」会让一个坏发版把所有 execution 永久钉在重试态，所以 `workflow_execution_timeout` 必须真的实现，而不只是有一个参数。

**代价**：D14 的一部分（M3 的默认重试上界来自引擎配置而非 `schedule_to_close_timeout`）。

**相关**：ADR-0003 · ADR-0012 · ADR-0013 · `concepts/determinism.md`
