# Retry 与 Failure

## Temporal 怎么做

**Activity 默认带 RetryPolicy**（由服务端动态配置提供）：`initial_interval` 1s、`backoff_coefficient` 2.0、`maximum_interval` = 100×initial、`maximum_attempts` 无限、`non_retryable_error_types` 空。重试只被 `schedule_to_close_timeout` 兜底；`maximum_attempts = 1` 即关闭重试。**Workflow Execution 默认不重试**（workflow 必须确定性，重跑整个 workflow 不会解决问题）。**WFT 不受 RetryPolicy 约束**。

四种 activity timeout，重试性截然不同：`schedule_to_start` **永远不可重试**（重试只会把它放回同一个队列）、`start_to_close` **永远可重试**、`schedule_to_close` 界定整体重试窗口、`heartbeat` 用于长任务续命（上限取 `start_to_close`）。

失败用 `Failure` proto 表达：`message` / `source` / `stack_trace` / `cause` / `encoded_attributes` + 一个 `oneof failure_info`。activity 里抛**任何**非 Temporal 错误都会被 SDK 转成 `ApplicationFailure { type: 错误类型名, non_retryable: false }`。`non_retryable_error_types` 与 `type` **精确匹配**；`next_retry_delay` 可以覆盖本次退避。

关键的反直觉事实：**activity 的失败重试不往历史里写事件**（Temporal 文档原话 "activity retries do not contribute events to Workflow History"）。

## 我们怎么做

- M3 就有默认重试：租约过期的 activity 任务由 scanner 写 `ActivityTaskTimedOut` 并重投 `attempt + 1`（间隔取退避）。
- P4 补齐 `schedule_to_start` / `start_to_close` / `schedule_to_close` / `heartbeat` + RetryPolicy 五字段 + `next_retry_delay` + `non_retryable` 精确匹配。
- wire 类型与 `RetryState` / `TimeoutType` 取值见 ADR-0013。

## 代价

D5：我们不做 transient —— 每次尝试都写进历史（换来「跑了两次」这件事肉眼可见）。D14：M3 的重试上界来自引擎配置，P4 才落成 `schedule_to_close`。
