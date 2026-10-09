# ADR-0015 里程碑重切与 gate

**决定**：里程碑与**两处挪动**：

1. scanner 与 `workflow_execution_timeout` 的**执行**从 P4 提到 **M3** —— M3 的 gate 是「`kill -9` 后无缝恢复」，workflow 侧的自愈路径就是「WFT 租约过期 → 写超时事件 → 重排」。
2. `ctx.patched` 从 P6 提到 **P5**（它复用 M1 的 `MarkerRecorded`，成本近乎为零）；`range_id` fencing 与 transient/buffered events 从 P5/P4 合并进 **P6 实验集**（P5 聚焦语义，P6 专做同语义不同实现的量化对比）。

**完成定义（gate）**：每个里程碑的完成 = `scripts/verify-mN.sh` **exit 0** + 该里程碑有可跑 demo + 文档同步（milestone 页，必要时新增 ADR）+ 打 git tag。

**为什么**：`verify-*.sh` 是把验收标准从散文变成**可执行事实**的唯一办法，它逼你在完成每一层时写下「怎么证明它成立」——这正是让「难度可控」可操作化的方式。而把 `patched` 提前，是因为它是整条确定性故事线的**兑现点**：前面所有关于重放、非确定性、WFT 失败重试的铺垫，到这里才第一次变成「我可以安全地改代码了」；把它压在最后一个阶段，等于把项目最有价值的结论藏起来。

**纪律**：不提前写下个里程碑的代码（见到就删）。这是「循序渐进」唯一可执行的形态。

**代价**：每层都要写脚本；P5 变重、P6 变轻。

**相关**：`DESIGN.md` §6 · `docs/milestones/`
