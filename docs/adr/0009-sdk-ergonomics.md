# ADR-0009 SDK 人体工学

**决定**：

```rust
#[workflow]                                                  // M3 起，M1/M2 用手写注册
async fn order(ctx: WorkflowContext, input: OrderInput) -> Result<OrderResult, WfError>
```

- `ctx` **按值传**（廉价 clone 句柄），业务代码可以随手把它 clone 进辅助函数。
- `ctx.execute_activity::<I, O>("charge_card", req, opts).await?` —— 错误类型 `ActivityFailure`，`?` 即失败 workflow（与 TS/Go 一致）。
- `ctx.sleep(dur)` · `ctx.now()` · `ctx.random::<T>()` · `ctx.uuid()` · `ctx.side_effect(|| …)` · `ctx.info()` · `ctx.is_replaying()` · `ctx.logger()`。
- 宏是**薄包装**：`#[workflow]` 生成类型擦除的调用器与注册表项。M1/M2 先用手写注册辅助函数，M3 引擎接口稳定后再上宏。

**为什么**：按值传消灭生命周期摩擦 —— 这正是「频繁借用 `ctx`」这一痛点的根治方式（`&WorkflowContext` 会让每个辅助函数都背上借用关系）。`ctx.logger()` 从 M1 就有，因为 `println!` 在教学代码里必然出现，而它会把单轮重放刷成不可读的日志。

宏推迟到 M3，是因为宏会**掩盖引擎接口的丑陋**：在还看不懂自己引擎的时候，你拿到的是一堆生成代码的报错。先手写，让编译错误落在真正写的代码上。

**代价**：`ActivityFailure` → `WfError` 的 `From` 转换是 `?` 能工作的前提，必须显式实现；`ctx.logger()` 需要在重放期静音（`is_replaying` 门控）。

**相关**：ADR-0003 · ADR-0013 · `concepts/workflow-task.md`
