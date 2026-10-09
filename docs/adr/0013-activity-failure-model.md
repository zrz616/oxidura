# ADR-0013 Activity 失败模型与重试分类

**决定**：wire 类型 `Failure { message, source, stack_trace, cause, failure_info }`，只实现四个 variant：`Application`、`Timeout`、`Canceled`、`Activity`（`Server` / `Reset` / `ChildWorkflowExecution` / `Nexus*` 不做）。

```rust
ApplicationFailureInfo { r#type: String, non_retryable: bool, details: Value, next_retry_delay: Option<Duration> }
TimeoutFailureInfo     { timeout_type: TimeoutType, last_heartbeat_details: Option<Value> }
ActivityFailureInfo    { scheduled_event_id, started_event_id, identity, activity_id, retry_state }
```

`RetryState` = `IN_PROGRESS` / `NON_RETRYABLE_FAILURE` / `TIMEOUT` / `MAXIMUM_ATTEMPTS_REACHED` / `RETRY_POLICY_NOT_SET` / `INTERNAL_SERVER_ERROR` / `CANCEL_REQUESTED`。
`TimeoutType` = `START_TO_CLOSE` / `SCHEDULE_TO_START` / `SCHEDULE_TO_CLOSE` / `HEARTBEAT`。

规则：

- activity 抛任何错误 → `ApplicationFailure { type: 错误类型名, non_retryable: false }`，包进 `ActivityFailure.cause`。`std::any::type_name::<E>()` 作兜底 type（**文档需警告它对重构敏感**）。
- `non_retryable_error_types` 对 `type` **精确匹配**（非子串）。
- **失败重试不写事件**（Temporal 原义）；超时写 `ActivityTaskTimedOut`。
- M3 实现默认重试（上界由引擎配置给，D14）；P4 实现三种 timeout + RetryPolicy 五字段 + `next_retry_delay` + heartbeat。

**为什么**：`non_retryable` 与 `type` 字符串是同一件事的两半 —— 让「错误类型名是契约的一部分」在 API 上显式可见，比隐式按错误种类分类更贴近 Temporal，也更容易解释「为什么改个名字就悄悄变成可重试了」。

**代价**：D5（每次尝试都进历史）、D14（M3 上界不是 `schedule_to_close_timeout`）。

**相关**：ADR-0007 · ADR-0012 · `concepts/retry-and-failure.md`
