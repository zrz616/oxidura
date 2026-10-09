# oxidura

Rust + 单个 PostgreSQL 的 Temporal 式工作流引擎（学习项目）。契约文档在 `docs/`。

- **开工前**：读 `docs/DESIGN.md` —— 范围、架构、数据模型、偏离表的唯一出处。
- **做某个里程碑**：读 `docs/milestones/` 里对应那一篇（`m0` `m1` `m2` `m3` `p4` `p5` `p6`）。每篇的 **Gate** 段就是完成定义，对应的 `scripts/verify-*.sh` 必须 exit 0。
- **改动 core 的公开类型**（`Event` / `Command` / `WorkflowContext` / `replay`）：先读 ADR 0002 / 0003 / 0005 / 0006 / 0010。这些形状是冻结的。
- **改 schema、提交路径、任务租约或时间触发**：读 ADR 0008 / 0011 / 0012。
- **需要对照 Temporal 原义**：读 `docs/concepts/<概念>.md`（每篇 = Temporal 怎么做 → 我们怎么做 → 代价；索引在 `DESIGN.md` §7）。
- **要解释或改动某处与 Temporal 的差异**：改 `docs/DESIGN.md` §5 偏离表里的 D 编号，别处只引用编号。
- **不确定某功能是否在范围内**：读 `docs/DESIGN.md` §2 的「永不做」清单。
- **做完一个里程碑**：跑 gate → 更新对应 `docs/milestones/<篇>.md` → 打 tag。

## Agent skills

### Issue tracker

Issues 与 spec 以 markdown 文件存在 `.scratch/<feature>/`（local markdown；长期看板是 Linear 团队 TOM，暂不接入，原因与接入条件见文件）。See `docs/agents/issue-tracker.md`.

### Triage labels

五个 canonical triage 角色沿用默认字符串，落在每个 issue 文件顶部的 `Status:` 行。See `docs/agents/triage-labels.md`.

### Domain docs

Single-context：词表在 `DESIGN.md` §1，决策在 `docs/adr/`，Temporal 对照在 `docs/concepts/`。See `docs/agents/domain.md`.
