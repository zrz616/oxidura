# Signals 与 Queries

## Temporal 怎么做

| | Signal | Query | Update |
|---|---|---|---|
| 方向 | 异步写 | 读 | 同步跟踪写 |
| 写历史 | 是（`WorkflowExecutionSignaled`） | **否** | 是（`UpdateAccepted`/`UpdateCompleted`） |
| 能阻塞 | — | **不能** | handler 能，validator 不能 |
| 能拿结果 | 否 | 同步返回 | 是 |

- **Query 不写历史、不产生 WFT、不能阻塞，且**可以打在**已关闭**的 workflow 上。它是唯一不产生事实的读路径。
- Signal 会触发新的 WFT。Go SDK 的语义是：**signal 送达时若没有对应的 channel，它就丢了**；`SignalWithStart` 是为了消掉「先发 signal 再启动」的竞态。
- Update 在写入前有 validator 把关，是唯一「服务端参与业务校验」的路径。

## 我们怎么做

- **Signal**：`WorkflowExecutionSignaled` 进事件子集，但它是 **AsyncEvent** —— 游标经过时不消费，投入 ctx 的按名缓冲；用户用拉式 `ctx.wait_signal("cancel").await` 取（ADR-0010）。
- **Query**：一行 `tasks(task_type = QUERY)`，**完成后删除、不产生任何事件**，结果写回行上的结果列由客户端轮询（D8）。执行它的 worker 必须重放历史并且**断言自己没有产出任何命令** —— 这条断言就是「重放是只读的」最直接的练习。
- **Update**：不在范围内（D10）。替代演示：signal 请求变更 → query 查状态 → 客户端轮询到结果。

## 代价

D13：未等待的 signal 我们**保留**（Temporal Go SDK 丢弃），语义更宽容，但代价是 signal 的消费非位置性 ⇒ 对非确定性的保护更弱。D8：query 依赖 worker 在跑，且以行为载体（Temporal 的服务端 query 分发路径我们未核实）。
