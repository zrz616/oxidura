# ADR-0001 范围与永不做

**决定**：覆盖到 P5 级语义（signal/query/child/CAN/cancel/patched）；「永不做」= Namespace 多租户、Update、Schedules、Nexus、Archival、WorkerVersioning、Local Activity、Search Attributes / Memo / Priority / Header。schema 不为它们留空列。

**为什么**：项目要同时满足「难度可控」与「不因简单而丢掉 Temporal 概念」。这两条的平衡点在于**区分「语义概念」与「分布式运维」**：前者（历史、重放、WFT、活动、timer、重试、signal、query、版本化）是引擎之所以是引擎的东西，后者（多租户、归档、跨集群复制、worker 版本下发）学不到工作流语义，只会把项目拖死。

⚠️ `Query` 必须留在范围内：它是唯一**不写历史**的读路径，是「历史 = 唯一真相」这条公理最好的反例。

**代价**：缺少 Update 的 validator 实践（唯一一处「服务端参与业务校验」的路径）；用 `concepts/signals-and-queries.md` 里的「signal + query 组合」替代演示。缺少 Local Activity，因此「活动调度开销」这一课只能靠 P6 基准间接体会。

**相关**：`DESIGN.md` §2 · ADR-0015
