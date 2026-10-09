# Replay

## Temporal 怎么做

worker 拿到一个 WFT 后，从**头**重放整个 workflow：历史里已发生过的操作（活动调用、timer、signal 等待）立刻返回记录过的结果，代码继续跑到「需要新进展」为止，然后产出命令。重放期间日志与拦截器副作用被抑制（Go 的 `workflow.IsReplaying`、TS 的 `unsafe.isReplaying`、Python 的 `unsafe.is_replaying()`）；官方警告：不要用重放状态去分支业务逻辑。

sdk-core（Rust）把历史喂给内部状态机，再向语言层发 `WorkflowActivation`；语言层负责调用 workflow 函数并把命令回填。缓存命中的 sticky worker 只拿**部分**历史（从 `previousStartedEventId + 1` 起）。空 WFT 会被折叠进上一个「逻辑 WFT」，重放时不会唤醒语言层。

## 我们怎么做

- **replay 边界** = 直到并包含本轮 `WorkflowTaskStarted` 的那段历史（ADR-0008）。它带来一个漂亮的推论：`ctx.now()` 直接取这条事件的 `event_time`，重放与 live 完全一致。
- 重放是**同步**推进：noop waker，poll 到 Pending，全程无 async runtime（ADR-0003）。
- 匹配是 demand-driven：每个 await 点主动要下一条期望事件，不匹配立刻带 `event_id` 报 `NonDeterminism`（ADR-0010）。
- `oxidura replay-check <run_id>` 拿真实历史重放一遍，是「改了 workflow 代码」之后的安全网。

## 代价

D3、D4：每轮 O(历史长度)。P6 用 sticky 缓存量化这个代价，而不是提前优化掉它。
