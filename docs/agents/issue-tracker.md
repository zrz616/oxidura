# Issue tracker: Local Markdown

Issues and specs for this repo live as markdown files in `.scratch/`.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The spec is `.scratch/<feature-slug>/spec.md`
- Implementation issues are one file per ticket at `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from `01`, never a single combined tickets file
- Triage state is recorded as a `Status:` line near the top of each issue file (see `triage-labels.md` for the role strings)
- Comments and conversation history append to the bottom of the file under a `## Comments` heading
- `<feature-slug>` 用 milestone 名（`m0` `m1` `m2` `m3` `p4` `p5` `p6`）或一个小功能的名字

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<feature-slug>/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or the issue number directly.

## `Status:` 的取值

五个 triage 角色（`needs-triage` / `needs-info` / `ready-for-agent` / `ready-for-human` / `wontfix`）只用于**待办**。
已完成并落地的条目写 `Status: done`，免得 `triage` 反复回访已经做完的东西。

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` (the Notes / Decisions-so-far / Fog body).
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every file it lists is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append a context pointer (gist + link) to the map's Decisions-so-far in `map.md`.

## 与 Linear 的关系：当前不接入，只记录意图

长期看板是 Linear 团队 **TOM**（workspace `tom-viner`）。这套技能**当前不驱动它**，原因和接入条件写在下面，免得下一个人重新调查一遍：

- 本 harness（omp）挂载的 Linear 工具**没有 issue 的增删查改**（create / read / update / assign 全缺），
  只有 labels / statuses / relations / comments / cycles / views / documents / initiatives / teams / users。
  于是 `to-spec` 的 publish 与 `triage` 的读写在这里会硬报「工具不存在」。
- **接入条件**：在能对 Linear 做 issue 增删查改的 harness 里跑这些技能（例如带完整 Linear MCP 的客户端），
  或者本 harness 挂上了那些工具。
- **接入时要做的三件事**：
  1. 把本文件改成 Linear tracker，并把下文的三条映射规则删掉；
  2. 在团队 TOM 里建五个 triage 标签（该团队目前一个标签都没有）——字符串见 `triage-labels.md`；
  3. 把 `.scratch/**` 里仍处于开放状态的条目搬过去。
- **迁移映射**：`.scratch/<feature>/spec.md` → 一个 Linear issue（正文 = spec）；
  `issues/NN-<slug>.md` → 它的 sub-issue；`Status:` 行 → issue 的状态 / 标签。
