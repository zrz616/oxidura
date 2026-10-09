# Issue tracker: Linear

Issues and specs for this repo live in Linear. Workspace `tom-viner`，团队 **TOM**（"Tom viner"），本仓库的工作都在 project **oxidura** 下。

## 固定坐标

| 东西 | 值 |
| --- | --- |
| team | `TOM` — id `dc1923fe-08cb-4638-9013-f0571234489b` |
| project（本仓库） | `oxidura` — id `b5852109-0561-40ad-9eac-201237826915` |
| 我（认领/指派用） | `lsslsd`（displayName `zrz616`）— id `4fc9e797-0288-4cc9-8024-b9ff32a58370` |
| 状态 | Backlog `c216178a-3736-4aef-a3f3-9b1acc981214` · Todo `e2c8ec11-5a97-4b7c-9a9a-ec935232d056` · In Progress `746fbf10-15ba-4687-92d3-8fe57c3aade8` · In Review `e7e2de4c-f774-4b7b-8a65-5b9f4babcf14` · Done `76522d09-6364-4e61-b7a6-2c50e535d9cf` · Canceled `35e83904-c43f-4bdc-a008-2d2d977dfa47` |
| 标签 | 见 `triage-labels.md` |

**约定**：一个仓库一个 project（`oxidura`），**不**按里程碑切 project —— M0…P6 的票都进同一个 project，靠标题前缀与标签区分。

⚠️ **同名陷阱**：团队 TOM 里还有**另一个项目**的 issue 叫 `M0：…` `M1：…` `M2：…` `M3：…`（TOM-5..8）。本仓库的票一律带 `oxidura` 前缀且挂在 project `oxidura` 下 —— **不要按标题里的 `M0/M1` 认票**，按 project 认。

## 常用操作

都用 omp 的 Linear 工具，不需要额外 CLI：

- **建 issue**：`linear_create_issue` —— 必给 `teamId` + `projectId` + `title` + `description`；`labelIds` 打标签、`stateId` 定状态、`parentId` 挂父 issue。
- **读 issue**：`linear_get_issue { issue: "TOM-12" }`。⚠️ 它**不返回关系**（阻塞边），见下。
- **列 issue**：`linear_list_issues { filter: { project: { id: { eq: "<oxidura project id>" } } }, first: 50 }`（已实测可用）。
- **搜索**：`linear_search_issues { term: "…" }`（可加 `includeComments`）。
- **改 issue**：`linear_update_issue` —— `title` / `description` / `stateId` / `addedLabelIds` / `removedLabelIds` / `assigneeId`。
- **留言**：`linear_create_comment`（读用 `linear_list_comments`）。
- **收口**：把 `stateId` 设为 Done —— 没有单独的 close 调用。
- **阻塞边（原生）**：`linear_create_issue_relation { issueId, relatedIssueId, type: "blocks" }` —— `issueId` 阻塞 `relatedIssueId`；反向的 "blocked by" 由 Linear 自己展示。
- **读阻塞边**：`linear_list_issue_relations` —— ⚠️ 它**没有** issueId 过滤参数，只能 `first`/`after` 全量拉，再按返回体里的 `issue.identifier` / `relatedIssue.identifier` 自己过滤。

## When a skill says "publish to the issue tracker"

建 Linear issue：`teamId` = TOM、`projectId` = oxidura，正文按该技能给的模板。一次 `/to-spec` = 一个 issue；一次 `/to-tickets` = 一票一个 issue（不要合并成一个文件或一张票）。

## When a skill says "fetch the relevant ticket"

`linear_get_issue { issue: "TOM-<n>" }`；要评论再补一次 `linear_list_comments`。

## Frontier（哪张票可以立刻开工）

两步，都能用工具直接查：

1. `linear_list_issues { filter: { project: { id: { eq: <oxidura> } } }, first: 50 }` → 留下状态不是 Done / Canceled 的票。
2. `linear_list_issue_relations { first: 50 }` → 找 `relatedIssue` 是这张票、`type = "blocks"` 的边，边里的 `issue` 就是它的 blocker。

一张票的**所有 blocker 都是 Done**（或它没有 blocker）→ 它就在前沿；多张时按状态时间/编号靠前的先做。已 `assignee` 的票视为被认领，跳过。

## Wayfinding operations（/wayfinder）

Linear 里没有「map 文件」，对应关系如下（`parentId` 由 `linear_create_issue` 的 schema 支持；本仓库尚未实际用过 `/wayfinder`，这一节是映射约定，不是实测记录）：

- **Map**：一个 issue（建议标题 `wayfinder: <effort>`），正文放 Notes / Decisions-so-far / Fog。
- **Child ticket**：用 `parentId` 挂在 map 下的 sub-issue，正文放那个问题；`wayfinder:<type>` 标签记类型（`research` / `prototype` / `grilling` / `task`）—— 这两个标签族**首次使用时才建**，别提前造。
- **Blocking**：原生 `blocks` 边（见上）。被 blocker 全部 Done 才算 unblocked。
- **Frontier**：map 的 open sub-issue 里，未被认领（`assignee` 为空）且未被阻塞的，按 map 顺序取第一个。
- **Claim**：`linear_update_issue { assigneeId: "4fc9e797-…" }` —— 动手前先指派。
- **Resolve**：`linear_create_comment` 写答案 → `stateId` 设 Done → 在 map 的 Decisions-so-far 里补一条指针（结论 + 链接）。

## 历史

本仓库曾用 local markdown 约定（`.scratch/<feature>/`）。2026-10-09 迁到 Linear：`.scratch/m0/spec.md` 变成 **TOM-14**（`oxidura M0：骨架（已完成）`），本地副本已删除。`.scratch/` 不再使用 —— 临时草稿放仓库外，正式结论写进 `docs/`。

本仓库目前**没有 git remote**，Linear 的 GitHub 集成也没接；如果以后要按 PR 做 triage，先接集成再改本文件。
