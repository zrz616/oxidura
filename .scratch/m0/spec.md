# Spec: M0 骨架

Status: done

> 已发布的 spec：tracker = local markdown（见 `docs/agents/issue-tracker.md`），本文件就是 issue 正文。
> M0 已落地并通过 gate **9/9**（commit `267a5fa`，tag `m0`），所以是 `done` 而不是 `ready-for-agent`。
>
> 设计权威始终是 `docs/DESIGN.md` 与 `docs/adr/`；这份文件只把 M0 转成 issue 形态。

## Problem Statement

作为这个学习项目的作者，我手里已经有一份自洽的设计（`docs/DESIGN.md` + 16 篇 ADR），但没有任何可运行的东西：

- 没有 crate 分层，所以「core 不认识 SQL、也不跑在 async runtime 上」这条设计约束只能靠自觉，几周内必然被 `use sqlx` 污染；
- 没有本地数据库，任何涉及 PostgreSQL 的结论都无法验证；
- 没有 gate，所以「这个里程碑做完了没有」只能靠感觉，文档里的承诺会慢慢和代码脱节；
- 没有 commit、没有 tag，后面的里程碑没有一个可以对比的基线。

## Solution

一个能 `cargo build` 的六 crate workspace、一个一条命令就能起停的常驻本地 PostgreSQL、一条把「做完」变成 exit code 的 gate 脚本，以及这批契约文档的落地。

完成定义、验收断言、演示命令都在 `docs/milestones/m0.md`；这份 spec 是它的 issue 形态。

## User Stories

1. 作为学习者，我想要六个 crate 的边界在第一天就存在，这样「core 不认识数据库」不是口头约定而是编译期事实。
2. 作为学习者，我想要 `cargo build` / `clippy -D warnings` / `fmt --check` / `nextest` 四条命令都能过，这样我在写第一行引擎代码之前就有一套不会互相吵架的工具链。
3. 作为学习者，我想要一条命令就能起停本地 PostgreSQL，这样验证存储层时不用去想 Docker，也不用手工敲 `pg_ctl`。
4. 作为学习者，我想要数据库的数据目录在仓库外，这样 `git status` 永远干净，也不会把数据文件提交进去。
5. 作为学习者，我想要开发库用非默认端口，这样 Postgres.app 自带的集群和我自己的集群不会互相踩。
6. 作为学习者，我想要 `DATABASE_URL` 只有一个出处，这样脚本、`.env`、未来的测试不会各说各话。
7. 作为学习者，我想要 gate 是脚本而不是文档里的自然语言段落，这样「这个里程碑做完了」是一个 exit code，而不是一种感觉。
8. 作为学习者，我想要 gate 里有一条直接对文件系统取证的检查（在 core 的 manifest 里搜数据库与 async runtime 依赖），这样架构约束由机器执行，而不是靠我记住。
9. 作为学习者，我想要 gate 检查契约文档还在（DESIGN + 16 ADR + 11 concepts + 7 milestones），这样误删文档会让 gate 变红。
10. 作为学习者，我想要 gate 检查所有脚本能在本机唯一的 bash 3.2 下解析，这样脚本不会因为写的人用了 4.x/5.x 构造而在别人的机器上死掉。
11. 作为学习者，我想要脚本能在本机真的跑起来（而不只是语法正确），这样「gate 通过」不依赖我对环境的记忆。
12. 作为学习者，我想要 `oxidura` 这个二进制从 M0 就存在，这样 M3 的 CLI 是在既有形状上长出来，而不是凭空冒出来。
13. 作为学习者，我想要 workspace lints 在 M0 就位（含 `clippy::large_enum_variant`），这样约束在我写出 `Event` 枚举之前就已经生效。
14. 作为学习者，我想要里程碑页记下落地时踩到的坑，这样下一个里程碑的脚本不会重踩一遍。
15. 作为学习者，我想要这次骨架被 commit 并打上附注 tag，这样后面每个里程碑都能与它对比。
16. 作为学习者，我想要 M0 明确声明「不做任何引擎逻辑、任何表、任何 CLI 子命令」，这样我不会在骨架里偷偷塞进一个半成品内核。
17. 作为维护者，我想要 gate 的每一项在失败时打印取自命令本身的证据（失败输出的尾部），这样被拦下来的人不需要二次实验。

## Implementation Decisions

- **六个 crate**：`oxidura-core`（零依赖）、`oxidura-store-postgres`、`oxidura-sdk`、`oxidura-sdk-macros`（`proc-macro = true`，必须独立 crate）、`oxidura-worker`（bin 名 `oxidura`）、`examples`；**目录平铺在仓库根**——因为 gate 是用相对路径对 `oxidura-core/Cargo.toml` 取证的（ADR-0004）。
- **根 manifest 是 virtual workspace**：`resolver = "3"`、`edition = "2024"`；package 元数据集中在 `[workspace.package]`，lint 在 `[workspace.lints]`，成员用 `version.workspace = true` 与 `[lints] workspace = true` 继承。
- **约束的执行者是 crate 边界 + gate**，不是注释：因此 core 的 manifest 里连「写这条规则的注释」都不能出现被搜的关键词，否则 gate 自己变红（这条在实现时真的踩到了，见下）。
- **`scripts/dev-pg.sh` 是本地集群的唯一出处**：Postgres.app 的 `pg_ctl` / `initdb` / `createdb` / `psql`；数据目录 `~/.local/share/oxidura/pgdata`（仓库外，可用 `OXIDURA_PGDATA` 覆盖）；端口 **55432**（避开 5432，可用 `OXIDURA_PGPORT` 覆盖）；`--auth=trust` 仅用于本机开发；子命令 `start` / `stop` / `status` / `url` / `psql` / `logs`，其中 `start` 幂等、首次自动 `initdb` 并建库。
- **`.env.example` 不复制**端口 / 用户 / 库名，只指向 `scripts/dev-pg.sh url`；gate 取 `DATABASE_URL` 时优先本地 `.env`，否则问脚本。
- **脚本形态**：`set -euo pipefail`；只用 macOS 自带 bash 3.2 能跑的构造；**变量名后紧跟中文时必须写 `${VAR}`**——bash 3.2 在 C locale 下会把后面的 UTF-8 字节并进变量名，`"（端口 $PGPORT）"` 直接报 unbound（这个坑在实现时命中了 `start`）。
- **完成定义** = `scripts/verify-m0.sh` 的 9 项断言：build / clippy `-D warnings` / `fmt --check` / `nextest --no-tests=pass` / `dev-pg.sh start` / `psql "$DATABASE_URL" -c 'select 1'` / core manifest 无引擎依赖 / `scripts/*.sh` 语法 / 契约文档齐全。
- **`examples` 是合法包名**，cargo 接受，所以 ADR-0004 的六个名字原样落地，没有改名。
- **收尾方式**：一次初始提交 + 附注 tag `m0`（本仓库此前无任何 commit）。

## Testing Decisions

- **M0 没有引擎行为可测，所以它唯一的缝就是 gate 脚本本身**：以黑盒方式调用 `scripts/verify-m0.sh`，只观察 exit code 和它打印的证据。M0 **不引入任何测试专用接口**。
- 不给 gate 写单测：验证「gate 会不会变红」需要构造一个故意错误的仓库树，收益低于成本。替代做法是让每个断言都打印取自命令本身的证据（失败输出的尾部 25 行），因为 gate 的读者是被它拦下来的人。
- 断言必须真的能变红，而不是恒真：例如 core 的依赖检查用 `grep`（不是 `grep -q`），命中的行本身就是诊断信息。
- 用 `nextest` 而非 `cargo test`（后续里程碑需要 per-test 隔离与更快的并发），零测试时必须 `--no-tests=pass`。
- 真库连通性不做 mock：M0 就要求一条真连接（`select 1`），因为「能连库」是 M2 全部结论的前提。
- **后续里程碑的缝（预先声明，供确认）**：M1 用 `replay()` + 内存 `TestWorkflowEnvironment` 作为最高缝（纯内存、零 IO）；M2 用真 PostgreSQL + store 的公开 API；M3 用两个进程 + `kill -9` 做真实恢复。

## Out of Scope

- 任何引擎逻辑：事件、命令、`replay()`、匹配器、确定性时钟。
- 任何表、migration、SQL 语句。
- 任何 CLI 子命令（连 `--help` 都没有）。
- 任何 async runtime、任何数据库驱动依赖（core 永远没有；其他 crate 到 M2/M3 才有）。
- 安装 `sqlx-cli`（M2 再说）。
- 多租户 / Namespace 之类的一切（见 `DESIGN.md` §2 的「永不做」）。

## Further Notes

- 环境事实：`/bin/bash` 3.2.57、`rustc 1.98.1`、`cargo-nextest 0.9.105`、PostgreSQL 来自 Postgres.app（无 Docker）。
- 已知未核实项（M0 不依赖它们）：见 `DESIGN.md` §5 的 D 编号与 `adr/0014` 的代价段。
- 这份 spec 与 `docs/milestones/m0.md` 是同一个东西的两种形态；发布到 tracker 后，里程碑页仍是仓库内的权威，issue 只承担「待办 / 结案」的角色。
