# ADR-0012 时间触发转移：scanner 是唯一出口

**决定**：一个后台循环（scanner）是**所有**「到点发生的事」的唯一出口，且这些转移各在一个事务里完成：

| 到点的事 | scanner 的动作 |
|---|---|
| WFT 租约过期 | 写 `WorkflowTaskTimedOut` + `WorkflowTaskScheduled` + 排新 WFT |
| activity 租约过期 | 写 `ActivityTaskTimedOut` + 重投 `attempt + 1`（间隔取退避） |
| `task_type = TIMER` 到期 | 写 `TimerFired` + `WorkflowTaskScheduled` + 排新 WFT |
| `workflow_execution_timeout` 到期 | 写 `WorkflowExecutionTimedOut`，关闭 execution |

`dequeue` **只取** `state = 'READY' AND visible_at <= NOW()`，绝不承认过期租约。scanner 自身用 `FOR UPDATE SKIP LOCKED`（多实例安全），周期可配（开发期 1s）。

**为什么**：懒式重投（让 dequeue 承认过期租约）会与 M3 的验收断言**直接矛盾** —— 我们要在历史里看到 `WorkflowTaskTimedOut` 与 `ActivityTaskTimedOut`，而懒式根本不会产生它们。懒式还会把两个概念糊在一起：你无法再解释「这次 activity 为什么跑了两次」。让 scanner 独占出口，语义才唯一：「`LEASE` → 过时 → 写超时事件 → `READY`」。

这条也补上了 ADR-0007 里 `workflow_execution_timeout` 的执行缺口：**参数没有执行者等于没实现**。

**代价**：任务要多等一个扫描周期（可配）。Temporal 里 timer 由 History 分片的 timer 队列处理，我们合并到一张表（D6）。

**相关**：ADR-0007 · ADR-0011 · `concepts/timers.md`
