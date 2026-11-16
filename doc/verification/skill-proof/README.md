# MarqdoProof — `marqdo prove` 生成代码的语义底座

`MarqdoProof.lean` 是生成 Lean 定理所依赖的全部符号（`Val`、`mq*` 辅助函数、
矛盾引理），无公理、无 `opaque`。发布/校验环境把它作为 lake 项目
`verification/skill-proof/` 安装（仓库 checkout 中该目录属 root，构建期
落在 `target/proof/skill-proof/`，两者内容一致）。

## 构建与校验（Lean 4.35.0）

```sh
export PATH=<lean-toolchain>/bin:$PATH   # lean 4.35.0 + lake 5.0.0
cd verification/skill-proof
lake build                                # 产出 MarqdoProof.olean
lake env lean <generated>.lean            # exit 0 且无 error ⇒ 该文件全部定理成立
```

## 生成文件的形状（emit.rs 契约）

- 命名空间强制 ASCII：`审批.mq.md` → `namespace Generated.u5BA1u6279_mq`
  （Lean 的 `namespace` 命令只接受 ASCII 名字）。
- 非 ASCII 标识符一律 guillemet：`def «审批单»`；guillemet 内 `:` 替换为 `_`
  （`«调用_人工审核»`）。`Op.name` 的字符串显示保持原始 id。
- 定理形如：
  ```lean
  theorem «审批单.大额必经人工» (env : Env) («金额» : Val) («等级» : Val)
      («h_前提» : mqTruth (mqGt «金额» (Val.int 10000))) :
      ∀ («结果» : Val) («轨迹» : List Op),
        ((«结果», «轨迹») = «审批单» env [] «金额» «等级») →
        mqTruth ((«轨迹».map Op.name).contains "人工审核") := by
    intro «结果» «轨迹» h_run
    unfold «审批单» at h_run
    try split at h_run
    all_goals
      simp_all [mqTruth, Op.name, mqAccess, mqResultOk, mqEq]
      try
        first
        | exact (mqGt_mqLe_false ‹mqGt _ (Val.int _) = true› ‹mqLe _ (Val.int _) = true›).elim
        | …
        | omega
  ```
- run 等式方向 `(结果, 轨迹) = run env [] …`：输出变量在左，simp_all 才能把
  输出替换进目标；run 从空轨迹起跑（§2：轨迹是语法产物，函数入口为空）。

## 实测结论（2026-11-16，Lean 4.35.0 / lake 5.0.0）

- 业务样例（审批.mq.md，3 条定理）：`lake env lean` 0 error，全部证明。
- 10 个测试夹具生成文件：9 个 0 error；`p2_len`（计数器）的
  `长度(轨迹) = 2` 为**假命题**（该函数不产生操作，轨迹恒空）——Lean 如实
  未证，符合"绝不 lean 无法证明的 marqdo 可以证明"。
