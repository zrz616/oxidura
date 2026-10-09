# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

Layout: **single-context**（没有 `GLOSSARY-MAP.md`，也不会有）。

## Before exploring, read these

- **`docs/DESIGN.md` §1 词表** —— 本仓库的词表在这里。**仓库里没有 `GLOSSARY.md`，别去找它**。
  §1 把 `replay 边界` / `lease` / `scanner` / `gate` / `seq` / `三类失败出口` / `D 编号` 定义一次，全篇当 token 复用。
- **`docs/DESIGN.md` §2「永不做」清单** —— 判断某个功能是否在范围内。
- **`docs/adr/`**（16 篇）—— 碰哪个区域就读哪几篇；`AGENTS.md` 的触发分支就是它的索引。
- **`docs/concepts/*.md`**（11 篇）—— 需要对照 Temporal 原义时读；每篇 = Temporal 怎么做 → 我们怎么做 → 代价。
- **`docs/milestones/*.md`** —— 做某个里程碑时读；每篇的 **Gate** 段就是完成定义。

## Use the glossary's vocabulary

输出里命名领域概念时（issue 标题、重构提案、假设、测试名），用 §1 词表里的词，不要漂移到它刻意避开的同义词：

- 「重放边界」就是 `replay 边界`，不要写成 "replay window" / "history range"。
- 失败只说三组词：execution failure / workflow-task failure / lease failure。
- 与 Temporal 的差异一律引用 `D 编号`（`DESIGN.md` §5），不要在其他地方重新解释一遍。
- 任务租约是 `lease`，承接它的表是 `tasks`，扫时间的那个东西叫 `scanner`。

要用的概念不在词表里，只有两种可能：要么在发明这个项目不用的说法（重新考虑），要么是真缺口
（记下来，用 `/domain-modeling` 补进 §1，而不是在本文件里另立一套）。

## Flag ADR conflicts

输出若与既有 ADR 矛盾，明确说出来，不要静默覆盖：

> _与 ADR-0003（同步 poll-to-quiescence）冲突，但值得重开，因为…_

两条本仓库特有的硬规矩：

- core 的公开类型（`Event` / `Command` / `WorkflowContext` / `replay`）的形状是**冻结**的：
  要改先改 ADR-0002 / 0003 / 0005 / 0006 / 0010 里对应的那一篇，而不是先改代码。
- 改 schema、提交路径、任务租约或时间触发，先读 ADR-0008 / 0011 / 0012。
