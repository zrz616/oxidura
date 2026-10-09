# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the actual label strings used in this repo's issue tracker.

| Canonical role | Label string in our tracker | Meaning |
| --- | --- | --- |
| `needs-triage` | `needs-triage` | Maintainer needs to evaluate this issue |
| `needs-info` | `needs-info` | Waiting on reporter for more information |
| `ready-for-agent` | `ready-for-agent` | Fully specified, ready for an AFK agent |
| `ready-for-human` | `ready-for-human` | Requires human implementation |
| `wontfix` | `wontfix` | Will not be actioned |

## 在本仓库里怎么落地

- tracker 是 local markdown，所以这些字符串**不是 GitHub 标签**，而是每个 issue 文件顶部 `Status:` 行的取值：

  ```markdown
  # Spec: M1 纯内存重放内核

  Status: ready-for-agent
  ```

- 五个角色字符串与默认完全一致（未做任何改写）。
- 已完成并落地的条目写 `Status: done` —— 它是终态，不属于五个 triage 角色。
- 将来若接入 Linear（团队 TOM，见 `issue-tracker.md`），就在 TOM 下建这五个同名标签，
  让 `triage` 复用而不是另造一套。
