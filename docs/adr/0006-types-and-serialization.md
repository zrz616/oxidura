# ADR-0006 类型形状、serde 与数据库表示

**决定**：

```rust
struct HistoryEvent { event_id: i64, event_time: DateTime<Utc>, attributes: Event }

#[serde(tag = "event_type", rename_all = "SCREAMING_SNAKE_CASE")]
enum Event { ActivityTaskScheduled(Box<ActivityTaskScheduledAttrs>), /* … */ }
```

`Command` 同构。DB 列：`event_type SMALLINT`（**Temporal 官方编号**）+ `event_time TIMESTAMPTZ` + `attributes JSONB`；`executions.status SMALLINT`（同样用官方编号，RUNNING = 1）。`Payload = serde_json::Value`，转换经 `PayloadConverter` trait（当前只有 Json 实现）。

**为什么**：serde 的内部 tag **就是 Temporal 的事件名**，落库后 `attributes` 肉眼可读、`attributes->>'activity_id'` 可查 —— 这是「充分利用 PostgreSQL」的最直接兑现。每个 variant 用 `Box` 包住属性结构体，避免单个 enum 被最大 variant 撑大（`clippy::large_enum_variant` 兜底）。用官方编号而非 TEXT，是为了让 DB 里的数字能直接与 `event_type.proto` / `workflow.proto` 对照。

**代价**：JSONB 会丢类型信息（i64 精度、`Duration` 往返），必须用单测覆盖；D2（无 codec / 加密 / 非 JSON 编码下）。

**相关**：ADR-0005 · ADR-0011 · `DESIGN.md` §5
