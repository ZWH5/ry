# Docker Hub 部署配置指南

本项目已配置 GitHub Actions 自动构建 Docker 镜像并推送到 Docker Hub。

## 配置步骤

### 1. 生成 Docker Hub Token

1. 登录你的 Docker Hub 账户 (superz5)
2. 前往 Account Settings → Security
3. 点击 "New Access Token"
4. 创建新的访问令牌
5. 复制生成的令牌

### 2. 添加 GitHub Secrets

在你的 GitHub 仓库中添加以下 Secrets：

1. 前往仓库设置 → Secrets and variables → Actions
2. 点击 "New repository secret"
3. 添加以下两个 Secret：

| Secret 名称 | 值 |
|-----------|---|
| `DOCKER_HUB_TOKEN` | 你的 Docker Hub 访问令牌 |
| `DOCKER_TOKEN` | 你的原有 Docker Hub 令牌（如果有） |

### 3. 工作流配置

GitHub Actions 工作流已配置为：

- **触发条件**：
  - 主分支的 push 事件
  - 标签 push 事件（用于发布）
  - Pull Request 事件（需要 "Run CI" 标记）

- **构建过程**：
  1. `build-backend`: 编译 Rust 后端（支持 x86_64 和 aarch64）
  2. `build-docker`: 构建和推送 Docker 镜像

- **镜像推送目标**：
  - Docker Hub: `docker.io/superz5/ryot`
  - GitHub Container Registry (GHCR): `ghcr.io/IgnisDa/ryot`

### 4. 镜像标签

生成的镜像标签包括：

- `develop`: 开发分支最新版本
- `latest`: 最新发布版本
- `v{version}`: 基于 Git 标签的版本号
- `sha-{commit-sha}`: 基于提交哈希

### 5. 第一次触发构建

简单推送代码到主分支即可触发自动构建：

```bash
git push myrepo main
```

构建完成后，镜像将自动推送到 `superz5/ryot` Docker Hub 仓库。

### 6. 监控构建过程

在 GitHub 上查看构建进度：

1. 前往 Actions 标签页
2. 查看 "Main" 工作流的执行状态
3. 点击具体的工作流运行查看详细日志

## 故障排除

### 构建失败

- 检查 GitHub Actions 日志查看具体错误
- 验证 Secrets 是否正确配置
- 确保 Docker Hub Token 有效

### 镜像未推送

- 验证 `DOCKER_HUB_TOKEN` Secret 是否设置
- 检查令牌是否有推送权限
- 确保 Docker Hub 账户未达到配额限制

## 手动触发构建

如果需要手动触发网站构建：

1. 前往 GitHub Actions
2. 选择 "Website" 工作流
3. 点击 "Run workflow" 按钮

## 更多信息

- 工作流配置文件: `.github/workflows/main.yml`
- Dockerfile: `./Dockerfile`
- 项目文档: `apps/docs/`
