# MacDevClean 开发说明书

## 1. 产品定位

`MacDevClean` 是一个面向 macOS 程序员的手动磁盘清理工具，用于扫描和清理开发过程中产生的缓存、构建产物、依赖缓存、临时文件、大模型缓存、协作软件大文件缓存等内容。

工具目标不是清理所有系统垃圾，而是帮助开发者快速找出“哪些东西占了大量磁盘空间”，并按照风险等级进行清理。

核心宗旨：

```text
只清理可再生成、可重新下载、不会破坏应用主数据、不会影响原应用正常运行的缓存和临时产物。
```

---

## 2. 使用方式

工具不需要每天跑，不需要后台常驻，不需要定时任务。

用户只在磁盘空间紧张时手动执行：

```bash
macdevclean scan
macdevclean clean
macdevclean clean --safe
macdevclean clean --deep
```

---

## 3. 风险等级设计

### Level 0：闭眼可删 / 默认安全清理

这类内容删除后，通常只影响下次构建、测试、索引速度，不影响应用运行，不影响项目源码，不影响账号数据。

可以默认勾选。

### Level 1：需要确认后删除

这类内容通常可以重新生成或重新下载，但删除后可能会导致：

```text
重新安装依赖
重新构建项目
重新下载包
重新索引
重新拉取模型
重新初始化缓存
```

不默认删除，需要用户确认。

### Level 2：高风险，不建议默认删除

这类内容删除后可能影响工作流，甚至丢失本地数据。

只扫描、提示、展示占用，不默认删除。
如要删除，必须二次确认。

### Level 3：禁止删除 / 不推荐工具处理

这类属于用户主数据、账号数据、证书密钥、系统数据、数据库数据，工具不应该提供删除能力。

---

# 4. 清理范围总览

## 4.1 闭眼可删：默认安全清理

| 类型           | 示例                                             | 说明                 |
| ------------ | ---------------------------------------------- | ------------------ |
| Python 字节码缓存 | `__pycache__`、`*.pyc`                          | Python 自动生成        |
| Python 测试缓存  | `.pytest_cache`、`.mypy_cache`、`.ruff_cache`    | 删除后测试/检查会重新生成      |
| 测试覆盖率产物      | `.coverage`、`htmlcov`、`coverage`               | 可重新生成              |
| 前端构建缓存       | `.next/cache`、`.turbo`、`.vite`、`.parcel-cache` | 不删源码               |
| 前端测试缓存       | `coverage`、`.nyc_output`                       | 可重新生成              |
| Go 构建/测试缓存   | `go clean -cache -testcache`                   | 不清 Go module cache |
| C/C++ 临时构建文件 | `CMakeFiles`、`.ninja_log`、`.ninja_deps`        | 不删源码               |
| IDE 日志       | VS Code、JetBrains、Xcode、Android Studio 日志      | 只清日志               |
| 临时文件         | `*.tmp`、`*.temp`、`.DS_Store`                   | 需限定扫描路径            |
| 旧安装包         | `~/Downloads/*.dmg`、`*.pkg`、`*.zip`            | 默认仅清 15/30 天以上     |

注意：即使是“闭眼可删”，也不能全盘乱扫。必须限定在用户配置的项目目录、缓存目录、下载目录中。

---

## 4.2 需要确认后删除

| 类型                        | 示例                                             | 风险                                                       |
| ------------------------- | ---------------------------------------------- | -------------------------------------------------------- |
| Node 依赖目录                 | `node_modules`                                 | 删除后需重新安装依赖                                               |
| Python 虚拟环境               | `.venv`、`venv`、`.tox`、`.nox`                   | 删除后需重建环境                                                 |
| pnpm store                | `pnpm store prune`                             | 删除未引用包，可能重新下载                                            |
| npm cache                 | `npm cache verify` / `npm cache clean --force` | 重新下载包                                                    |
| uv cache                  | `uv cache clean`                               | 重新下载 Python 包，uv 官方支持清理缓存。([uv][1])                      |
| Conda 缓存                  | `conda clean --all`                            | 删除包缓存和 tarballs，conda 官方用于移除未使用包和缓存。([docs.conda.io][2]) |
| Go module cache           | `go clean -modcache`                           | 删除后重新下载依赖                                                |
| Gradle cache              | `~/.gradle/caches`                             | 重新下载依赖                                                   |
| Maven repo                | `~/.m2/repository`                             | 重新下载依赖                                                   |
| Xcode DerivedData         | `~/Library/Developer/Xcode/DerivedData`        | 重新编译和索引                                                  |
| Android build cache       | `.gradle`、`build`                              | 重新构建                                                     |
| Rust Cargo cache          | `~/.cargo/registry`、`~/.cargo/git`             | 重新下载依赖                                                   |
| Flutter cache             | `.dart_tool`、`build`、`.pub-cache`              | 重新下载/构建                                                  |
| Playwright/Cypress 浏览器缓存  | `~/Library/Caches/ms-playwright`               | 重新下载浏览器                                                  |
| Docker build cache        | `docker builder prune`                         | 重新构建镜像                                                   |
| Docker stopped containers | `docker container prune`                       | 删除已停止容器                                                  |
| Homebrew 缓存               | `brew cleanup`                                 | Homebrew 官方也会周期性 cleanup。([docs.brew.sh][3])             |
| 协作软件大文件缓存                 | 飞书、企业微信、钉钉、微信附件                                | 历史附件可能需重新下载                                              |
| AI 模型缓存                   | HuggingFace、ModelScope、Torch、Ollama 等          | 重新下载模型，可能几十 GB                                           |

---

## 4.3 高风险，不建议默认删除

| 类型                        | 示例                                   | 原因                                                                          |
| ------------------------- | ------------------------------------ | --------------------------------------------------------------------------- |
| Docker volumes            | MySQL、Postgres、Redis 数据              | 可能是真实数据库数据                                                                  |
| Docker unused images 全量删除 | `docker system prune -a`             | 可能删除常用但当前未运行镜像                                                              |
| Docker volumes prune      | `docker system prune -a --volumes`   | Docker 官方说明 volumes 默认不会删除，需显式加 `--volumes`，因为可能包含数据。([docs.docker.com][4]) |
| Conda envs                | `~/miniconda3/envs/*`                | 完整开发环境                                                                      |
| Xcode Archives            | `~/Library/Developer/Xcode/Archives` | 历史打包记录                                                                      |
| iOS Simulator 数据          | `~/Library/Developer/CoreSimulator`  | 可能有测试数据                                                                     |
| Android Emulator 数据       | `~/.android/avd`                     | 模拟器用户数据                                                                     |
| Ollama models             | `~/.ollama/models`                   | 删除后需重新 pull 模型                                                              |
| LM Studio models          | 本地大模型目录                              | 用户主动下载资产                                                                    |
| ComfyUI / SD models       | `checkpoints`、`loras`、`vae`          | 用户模型资产                                                                      |
| 虚拟机磁盘                     | Docker Desktop、Colima、Lima、UTM       | 可能包含完整系统数据                                                                  |
| 本地数据库目录                   | MySQL/Postgres/MongoDB 数据目录          | 真实业务数据                                                                      |

---

## 4.4 禁止删除 / 不推荐工具处理

工具不应该提供删除能力：

```text
~/.ssh
~/.gnupg
~/.kube
~/.aws
~/.azure
~/.config/gcloud
~/Documents
~/Desktop，除非明确匹配安装包规则
~/Library/Keychains
~/Library/Application Support 中的未知应用数据
系统目录 /System /Library /usr /bin /sbin /etc /var
项目源码目录
.git
.env
.env.local
*.pem
*.key
*.p12
*.mobileprovision
数据库文件
聊天记录数据库
sqlite/db/wal/shm 文件
terraform.tfstate
```

---

# 5. 功能模块设计

## 5.1 开发构建缓存模块

覆盖语言和框架：

```text
Node.js / 前端
Python
Go
C / C++
Java / Gradle / Maven
Rust / Cargo
Flutter / Dart
Swift / iOS
Android
微信小程序 / uni-app / Taro
Bazel / Buck / Pants
Terraform / Pulumi
```

扫描规则：

```text
node_modules
.next/cache
.nuxt
.output
.turbo
.vite
.parcel-cache
.svelte-kit
dist
build
coverage
.nyc_output

__pycache__
.pytest_cache
.mypy_cache
.ruff_cache
.coverage
htmlcov
.venv
venv
.tox
.nox

cmake-build-*
CMakeFiles
CMakeCache.txt
.ninja_log
.ninja_deps

.gradle
target
.m2/repository
.cargo/registry
.cargo/git
.dart_tool
.pub-cache
Pods
Carthage/Build
bazel-*
buck-out
.pants.d
.terraform
```

---

## 5.2 包管理器缓存模块

优先使用官方命令，不直接删除内部目录。

支持：

```bash
npm cache verify
npm cache clean --force
pnpm store prune
yarn cache clean
bun pm cache rm
uv cache clean
pip cache purge
poetry cache clear --all pypi
conda clean --all
go clean -cache -testcache
go clean -modcache
brew cleanup
brew autoremove
```

---

## 5.3 Docker / 容器模块

扫描命令：

```bash
docker system df
docker images
docker ps -a
docker volume ls
```

清理命令：

```bash
docker builder prune
docker container prune
docker image prune
docker network prune
docker system prune
```

高风险命令：

```bash
docker system prune -a
docker system prune -a --volumes
docker volume prune
```

设计要求：

```text
Docker build cache：confirm
Stopped containers：confirm
Dangling images：confirm
Unused images：confirm / dangerous
Volumes：dangerous
```

---

## 5.4 IDE / 编辑器缓存模块

支持：

```text
VS Code
Cursor
Windsurf
JetBrains：IntelliJ / WebStorm / PyCharm / GoLand / CLion
Android Studio
Xcode
```

可清理：

```text
日志
临时文件
崩溃报告
插件下载缓存
旧版本缓存
索引缓存
```

不清理：

```text
配置文件
插件配置
账号登录信息
workspace 配置
keymap
settings.json
```

---

## 5.5 协作软件大文件模块

支持：

```text
企业微信 / WeCom
飞书 / Lark
钉钉 / DingTalk
微信
QQ / TIM，可选
Slack，可选
Discord，可选
Telegram，可选
```

扫描内容：

```text
图片
视频
语音
压缩包
PDF
Office 文件
安装包
临时下载文件
日志
会议录制缓存
```

默认规则：

```text
只扫描单文件 > 10MB
只推荐清理 30 天以上文件
删除前展示应用、类型、大小、路径、最后修改时间
默认移动到废纸篓
不直接永久删除
```

禁止删除：

```text
聊天数据库
账号配置
sqlite
db
wal
shm
plist
keychain
cookies
login data
```

用户提示：

```text
删除后不影响应用启动和正常聊天；
但历史聊天里的图片、视频、附件可能需要重新下载；
如果服务端文件已过期，可能无法重新打开。
```

---

## 5.6 AI / 本地模型缓存模块

支持：

```text
Hugging Face
ModelScope
PyTorch / Torch Hub
TensorFlow / Keras
Ollama
LM Studio
ComfyUI
Stable Diffusion WebUI
Whisper
vLLM
SGLang
Xinference
llama.cpp / GGUF
```

常见路径：

```text
~/.cache/huggingface
~/.cache/huggingface/hub
~/.cache/huggingface/datasets
~/.cache/modelscope
~/.cache/torch
~/.cache/torch/hub
~/.cache/torch/checkpoints
~/.keras
~/.ollama
~/.ollama/models
~/Library/Application Support/LM Studio
~/ComfyUI/models
~/ComfyUI/output
~/ComfyUI/temp
~/stable-diffusion-webui/models
~/Models
~/AI
~/ai-models
~/Downloads
```

Hugging Face 默认缓存一般在 `~/.cache/huggingface/hub`，也可通过 `HF_HOME` 或 `cache_dir` 修改。([huggingface.co][5])
Ollama macOS 官方说明 `~/.ollama` 包含模型和配置。([docs.ollama.com][6])

识别文件类型：

```text
*.safetensors
*.bin
*.pt
*.pth
*.ckpt
*.onnx
*.gguf
*.ggml
*.tflite
*.mlmodel
*.mlpackage
*.arrow
```

风险规则：

```text
模型日志：safe
临时下载残留：safe
ComfyUI temp/output：confirm
Hugging Face hub cache：confirm
ModelScope cache：confirm
Torch checkpoints：confirm
Ollama models：dangerous
LM Studio models：dangerous
ComfyUI checkpoints/loras/vae：dangerous
用户 ~/Models 目录：dangerous
```

建议额外能力：

```text
模型重复检测
模型迁移到外置硬盘
生成环境变量配置建议
```

示例：

```bash
export HF_HOME=/Volumes/ExternalSSD/AIModels/huggingface
export MODELSCOPE_CACHE=/Volumes/ExternalSSD/AIModels/modelscope
export TORCH_HOME=/Volumes/ExternalSSD/AIModels/torch
```

---

# 6. 命令设计

```bash
macdevclean scan
macdevclean scan --json
macdevclean scan --category dev
macdevclean scan --category ai
macdevclean scan --category chat
macdevclean scan --category docker

macdevclean clean
macdevclean clean --safe
macdevclean clean --deep
macdevclean clean --dry-run
macdevclean clean --trash
macdevclean clean --permanent

macdevclean ai scan
macdevclean ai duplicates
macdevclean ai migrate --target /Volumes/ExternalSSD/AIModels

macdevclean docker scan
macdevclean docker clean

macdevclean config init
macdevclean config edit
macdevclean history
macdevclean doctor
```

---

# 7. 配置文件设计

路径：

```bash
~/.macdevclean/config.yaml
```

示例：

```yaml
projectRoots:
  - ~/workspace
  - ~/Projects
  - ~/Code
  - ~/Developer

defaultMode: safe

deleteStrategy:
  default: trash
  permanentForSafeCache: true

downloads:
  enabled: true
  olderThanDays: 30
  minFileSizeMB: 50
  extensions:
    - .dmg
    - .pkg
    - .zip
    - .tar.gz
    - .tgz
    - .iso
    - .ipa
    - .apk

collaborationApps:
  enabled: true
  olderThanDays: 30
  minFileSizeMB: 10
  apps:
    - wecom
    - feishu
    - dingtalk
    - wechat

aiModels:
  enabled: true
  scanOnlyByDefault: true
  detectDuplicates: true
  customDirs:
    - ~/Models
    - ~/AI
    - ~/ai-models

rules:
  node:
    nodeModules: confirm
    buildCache: safe

  python:
    pycache: safe
    venv: confirm
    uvCache: confirm

  docker:
    builderCache: confirm
    stoppedContainers: confirm
    unusedImages: confirm
    volumes: dangerous

  xcode:
    derivedData: confirm
    archives: dangerous
    simulators: dangerous

exclude:
  - ~/.ssh
  - ~/.gnupg
  - ~/.kube
  - ~/Documents
  - ~/workspace/company-important-project
```

---

# 8. 扫描结果数据结构

```json
{
  "id": "node_modules:/Users/alex/workspace/app/node_modules",
  "category": "dev-dependency",
  "name": "node_modules",
  "path": "/Users/alex/workspace/app/node_modules",
  "sizeBytes": 12582912000,
  "risk": "confirm",
  "action": "delete-directory",
  "deleteStrategy": "trash",
  "reason": "Node.js dependency directory can be recreated by npm install / pnpm install.",
  "lastModified": "2026-06-10T10:20:00Z",
  "selectedByDefault": false
}
```

---

# 9. 安全删除规则

删除前必须执行：

```text
1. 展开 ~ 为真实路径
2. 解析 symlink
3. 检查是否命中禁止目录
4. 检查是否在扫描结果列表中
5. 检查风险等级
6. dangerous 项必须二次确认
7. 默认优先移动到废纸篓
8. 生成删除日志
```

禁止删除路径：

```text
/
~
/Users
/Users/当前用户名
/System
/Library
/usr
/bin
/sbin
/etc
/var
/private
/Applications
~/Documents
~/Library/Keychains
~/.ssh
~/.gnupg
~/.kube
```

危险确认方式：

```text
你即将删除高风险内容：Docker volumes
这可能包含数据库数据。
请输入 DELETE 确认：
```

---

# 10. 交互流程

## 10.1 扫描

```text
macdevclean scan
```

输出：

```text
MacDevClean 扫描完成

闭眼可删：
- Python 缓存                 1.2 GB
- 前端构建缓存                3.4 GB
- Go 构建/测试缓存             0.8 GB
- 旧安装包                    5.6 GB

需要确认：
- node_modules               18.7 GB
- Xcode DerivedData           9.2 GB
- Docker build cache         12.4 GB
- pnpm store                  6.1 GB
- 协作软件大文件              8.3 GB
- AI 模型缓存                42.8 GB

高风险，仅提示：
- Docker volumes             24.5 GB
- Ollama models              31.2 GB
- iOS Simulator 数据          16.4 GB

预计安全可释放：11.0 GB
确认后可释放：97.5 GB
高风险占用：72.1 GB
```

## 10.2 清理

```text
macdevclean clean
```

交互：

```text
请选择要清理的内容：

[x] Python 缓存                    1.2 GB   safe
[x] 前端构建缓存                   3.4 GB   safe
[x] 旧安装包                       5.6 GB   safe

[ ] node_modules                  18.7 GB   confirm
[ ] Xcode DerivedData              9.2 GB   confirm
[ ] Docker build cache            12.4 GB   confirm
[ ] AI 模型缓存                   42.8 GB   confirm

[ ] Docker volumes                24.5 GB   dangerous

预计释放：10.2 GB

是否执行？ y/N
```

---

# 11. 清理报告

每次清理生成：

```text
~/.macdevclean/reports/2026-06-19-153000.json
~/.macdevclean/reports/2026-06-19-153000.md
```

报告内容：

```text
清理时间
清理模式
总释放空间
成功项
失败项
跳过项
高风险项确认记录
删除策略：trash / permanent
```

---

# 12. MVP 开发范围

首版建议只做最有价值的部分：

```text
1. CLI
2. scan
3. clean --safe
4. clean --deep
5. dry-run
6. 配置文件
7. JSON / Markdown 报告
8. 开发缓存扫描
9. 包管理器缓存命令
10. Docker 扫描和安全清理
11. Downloads 安装包扫描
12. 协作软件大文件扫描
13. AI 模型缓存扫描
```

首版可以先不做：

```text
菜单栏 App
自动定时任务
复杂 UI
全量重复文件哈希
自动迁移模型
高风险项真实删除
```

---

# 13. 推荐技术方案

建议语言：

```text
Go
```

原因：

```text
单二进制分发
扫描文件快
适合写 CLI
跨平台可扩展
macOS 上部署简单
```

推荐依赖：

```text
cobra：CLI 命令
survey / bubbletea：交互式选择
yaml.v3：配置文件
fatih/color：终端颜色
```

目录结构：

```text
macdevclean
├── cmd
│   ├── root.go
│   ├── scan.go
│   ├── clean.go
│   ├── config.go
│   └── history.go
├── internal
│   ├── scanner
│   │   ├── scanner.go
│   │   ├── dev.go
│   │   ├── node.go
│   │   ├── python.go
│   │   ├── docker.go
│   │   ├── ai.go
│   │   ├── chat.go
│   │   └── downloads.go
│   ├── planner
│   │   ├── risk.go
│   │   └── plan.go
│   ├── executor
│   │   ├── executor.go
│   │   ├── shell.go
│   │   ├── trash.go
│   │   └── guard.go
│   ├── report
│   │   ├── json.go
│   │   └── markdown.go
│   └── config
│       └── config.go
├── pkg
├── README.md
└── go.mod
```

---

# 14. 验收标准

首版验收标准：

```text
1. scan 能展示各类缓存占用大小。
2. safe 项可以一键清理。
3. confirm 项必须用户确认。
4. dangerous 项默认不清理。
5. 禁止路径无法删除。
6. 支持 dry-run。
7. 清理后生成报告。
8. 删除失败不能中断整个任务。
9. 对聊天软件和 AI 模型只做谨慎扫描，不默认删除。
10. 所有删除行为都可追溯。
```

---

# 15. 给 AIDE 的最终开发提示词

```text
请开发一个 macOS 程序员专用磁盘清理工具，名称为 MacDevClean。

工具定位：
这是一个手动触发的开发者磁盘清理工具，不需要后台常驻，不需要定时任务。核心宗旨是只清理可再生成、可重新下载、不会破坏应用主数据、不会影响原应用正常运行的缓存、构建产物、临时文件、大文件附件缓存和模型缓存。

请用 Go 实现 CLI 工具。

核心命令：
- macdevclean scan
- macdevclean scan --json
- macdevclean clean
- macdevclean clean --safe
- macdevclean clean --deep
- macdevclean clean --dry-run
- macdevclean config init
- macdevclean history
- macdevclean ai scan
- macdevclean ai duplicates

风险等级：
1. safe：闭眼可删，默认勾选。
2. confirm：需要用户确认后删除。
3. dangerous：高风险，默认只扫描，不建议删除，删除必须二次输入 DELETE。
4. forbidden：禁止删除。

必须支持的清理范围：
1. 开发构建缓存：Node.js、Python、Go、C/C++、Java、Gradle、Maven、Rust、Flutter、Swift、Android、微信小程序、Bazel、Terraform。
2. 包管理器缓存：npm、pnpm、yarn、bun、pip、uv、poetry、conda、go、brew。
3. Docker 缓存：builder cache、stopped containers、dangling images、unused images；Docker volumes 只提示为 dangerous。
4. IDE 缓存：VS Code、Cursor、JetBrains、Android Studio、Xcode 的日志、临时文件、索引缓存。
5. 协作软件大文件：企业微信、飞书、钉钉、微信等，只扫描图片、视频、附件、日志、临时下载文件，不删除聊天数据库和账号数据。
6. AI 模型缓存：Hugging Face、ModelScope、PyTorch、TensorFlow/Keras、Ollama、LM Studio、ComfyUI、Stable Diffusion、Whisper、vLLM、SGLang、Xinference、本地 GGUF/safetensors 模型。

安全要求：
1. 禁止删除 /、~、/System、/Library、/Applications、~/Documents、~/.ssh、~/.gnupg、~/.kube、Keychains 等敏感路径。
2. 不删除项目源码、.git、.env、证书、密钥、数据库、terraform state、聊天记录数据库。
3. 删除前必须展示路径、大小、类型、风险等级和原因。
4. 支持 dry-run。
5. 对用户可见文件、聊天附件、大模型文件，默认移动到废纸篓，不直接永久删除。
6. 每次清理生成 JSON 和 Markdown 报告。

首版优先完成 scan、clean --safe、clean --deep、dry-run、报告和配置文件。
```

这份文档已经把“闭眼可删 / 需要确认 / 高风险不建议删 / 根本不推荐删”分开了。按这个边界做，工具会比较稳，不会变成那种看起来清得很爽、但把开发环境和数据一起干掉的危险工具。

[1]: https://docs.astral.sh/uv/concepts/cache/?utm_source=chatgpt.com "Caching"
[2]: https://docs.conda.io/projects/conda/en/stable/commands/clean.html?utm_source=chatgpt.com "conda clean"
[3]: https://docs.brew.sh/FAQ?utm_source=chatgpt.com "FAQ (Frequently Asked Questions)"
[4]: https://docs.docker.com/reference/cli/docker/system/prune/?utm_source=chatgpt.com "docker system prune"
[5]: https://huggingface.co/docs/huggingface_hub/en/guides/manage-cache?utm_source=chatgpt.com "Understand caching"
[6]: https://docs.ollama.com/macos?utm_source=chatgpt.com "macOS"
