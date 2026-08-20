# Gemini Persona — 并行分析策略

## 同时审查多个文件的最佳实践

### 1. 并行分析模板

```
Task: Review the following files for consistency and issues

=== File 1: src/services/AuthService.ts ===
=== File 2: src/services/UserService.ts ===
=== File 3: src/services/PaymentService.ts ===

For EACH file, report:
1. 是否符合项目约定
2. 是否有安全问题
3. 是否可以优化

Then provide a CROSS-FILE analysis:
1. 是否有重复逻辑
2. 接口是否一致
3. 依赖关系是否合理
```

### 2. 并行 vs 串行决策

| 场景 | 策略 | 原因 |
|------|------|------|
| 审查 3+ 个独立文件 | 并行 | 文件间无依赖 |
| 跟踪调用链 | 串行 | 需要上一个文件的结果 |
| 重构影响分析 | 先并行扫描后串行深入 | 先广度后深度 |
| 代码库概览 | 并行 | 快速建立全局视图 |

### 3. 多文件协同修改

```
1. 全局扫描 → 识别所有需要修改的位置
2. 列出修改清单（按依赖排序）
3. 逐个修改（从底层依赖到上层调用者）
4. 交叉验证所有修改的一致性

Key insight: Gemini 的超长上下文让它能一次看到所有需要改的地方，
避免遗漏跨文件的影响。
```

### 4. 大代码库的并行子任务

```
对大型项目的分析策略：
  Sub-task 1: API 层分析（route 定义、middleware、错误处理）
  Sub-task 2: Service 层分析（业务逻辑、数据变换、外部调用）
  Sub-task 3: Data 层分析（ORM 模型、查询、迁移）
  Sub-task 4: Test 层分析（覆盖率、测试质量、边界条件）
  
  每层独立完成 → 汇总层间接口检查 → 整合报告
```
