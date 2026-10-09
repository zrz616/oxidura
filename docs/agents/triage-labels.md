# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the actual label strings used in this repo's issue tracker.

| Canonical role | Label string in Linear | Label id | Meaning |
| --- | --- | --- | --- |
| `needs-triage` | `needs-triage` | `144509b9-301d-4d27-86d6-b97d52cfaee9` | Maintainer needs to evaluate this issue |
| `needs-info` | `needs-info` | `c188bcc6-24c2-42f3-823d-cfa6b88a5018` | Waiting on reporter for more information |
| `ready-for-agent` | `ready-for-agent` | `005addf4-f14a-4799-8e21-48aff284fbfe` | Fully specified, ready for an AFK agent |
| `ready-for-human` | `ready-for-human` | `64ea5e9e-70d8-4939-827d-24b1fce8f53a` | Requires human implementation |
| `wontfix` | `wontfix` | `2ec205b9-b1bb-4b77-ae61-8ad4a00257c9` | Will not be actioned |

## 怎么用

- 五个标签**已经在团队 TOM 里建好**（字符串与默认一致），所以 `triage` 直接复用它们，不要另造同名或近义标签。
- 打/去标签：`linear_update_issue { addedLabelIds: [...] }` / `{ removedLabelIds: [...] }`；新建 issue 时直接给 `linear_create_issue` 的 `labelIds`。
- 状态与标签是两回事：**标签**表达分类（上表），**状态**表达进度（Backlog / Todo / In Progress / In Review / Done / Canceled）。`ready-for-agent` 的票默认落在 Todo，被认领时改 In Progress 并 `assigneeId`。
- 已交付并结案的票：**不要**留 `ready-for-agent`，把状态设为 Done 即可（例：TOM-14）。

## 与 `.scratch/` 的关系

本仓库早期版本用 local markdown，那时这五个字符串是 issue 文件顶部 `Status:` 行的取值。现在 tracker 是 Linear，`Status:` 约定作废 —— 见 `issue-tracker.md`。
