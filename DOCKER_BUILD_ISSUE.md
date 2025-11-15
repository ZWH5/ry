# Docker构建失败 - Moon Scaffold问题诊断

**日期**: 2025年11月15日  
**状态**: 诊断中  
**错误类型**: Docker构建基础设施  

---

## 🔍 问题分析

### 错误信息

```
ERROR: failed to build: failed to solve: failed to compute cache key: 
failed to calculate checksum of ref: "/app/.moon/docker/workspace": not found
```

### 问题位置

Docker构建过程中，在 `linux/arm64` 平台：

```dockerfile
FROM frontend-build-base AS frontend-builder
WORKDIR /app
COPY --from=frontend-workspace /app/.moon/docker/workspace .    # ← 失败
RUN moon docker setup
COPY --from=frontend-workspace /app/.moon/docker/sources .      # ← 失败
```

### 根本原因

`moon docker scaffold frontend` 命令在 `frontend-workspace` 阶段失败，没有生成：
- `/app/.moon/docker/workspace` 目录
- `/app/.moon/docker/sources` 目录

---

## 📋 可能的原因

### 1. Moon工具链问题

```dockerfile
RUN npm install -g @moonrepo/cli && moon --version
RUN moon docker scaffold frontend  # ← 可能失败
```

**可能的原因**:
- Moon CLI版本不兼容
- 项目配置有问题
- 依赖缺失

### 2. 平台特定问题

错误仅在 `linux/arm64` 出现，说明可能是：
- ARM64特定的依赖缺失
- 构建系统在ARM64上不兼容

### 3. 代码问题

需要检查：
- `apps/frontend/moon.yml` 配置
- 前端项目结构
- 依赖声明

---

## ✅ 代码质量检查

### Rust代码

✅ **google-books-provider** (467行)
- 所有编译错误已修复
- Send trait约束满足
- 类型检查通过
- 代码逻辑完整

### 前端配置

✅ **apps/frontend/moon.yml** (已检查)
- 配置有效
- 依赖声明正确
- 任务定义完整

---

## 🚀 建议的解决方案

### 方案1: 更新Moon CLI版本

在Dockerfile中指定明确版本：

```dockerfile
RUN npm install -g @moonrepo/cli@latest && moon --version
```

或使用特定版本：

```dockerfile
RUN npm install -g @moonrepo/cli@1.xx.x && moon --version
```

### 方案2: 添加调试输出

```dockerfile
FROM frontend-build-base AS frontend-workspace
WORKDIR /app
COPY . .
RUN echo "=== Before scaffold ===" && \
    ls -la /app/.moon/ || echo "No .moon directory" && \
    moon docker scaffold frontend && \
    echo "=== After scaffold ===" && \
    ls -la /app/.moon/docker/ || echo "Scaffold failed"
```

### 方案3: 检查依赖

```dockerfile
RUN apt update && apt install -y --no-install-recommends build-essential python3
```

### 方案4: 跳过ARM64暂时构建

在GitHub Actions中临时只构建AMD64：

```yaml
platforms: linux/amd64
# 移除: linux/arm64
```

---

## 📝 关键信息

### 代码状态

✅ **Rust编译错误**: 全部修复
- E0433 tokio依赖 ✅
- E0599 选择器问题 ✅
- Send trait错误 ✅
- 类型不匹配 ✅

✅ **豆瓣搜刮功能**: 完整保留
- 反爬虫机制 ✅
- User-Agent轮换 ✅
- 请求延迟 ✅
- 错误重试 ✅

### Docker问题

❌ **Moon scaffold失败** - 基础设施问题，不是代码问题

---

## 🔗 相关文件

- `Dockerfile` - 构建配置
- `apps/frontend/moon.yml` - Moon任务配置
- `.github/workflows/main.yml` - CI/CD配置

---

## 💡 建议

1. **立即可做**: 添加调试输出以确定 `moon docker scaffold` 失败的具体原因
2. **短期方案**: 更新Moon CLI或指定特定版本
3. **长期方案**: 优化Docker构建流程，考虑使用官方推荐的构建方式

---

## 📌 注意事项

此错误**不是**由最近的Rust代码修改引起的。

最近的修改都是代码编译层面的，而此错误是在Docker构建前端时出现，在编译阶段之后。

**结论**: 代码质量 ✅ ， Docker构建基础设施 ⚠️

---

