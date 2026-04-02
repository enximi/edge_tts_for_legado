# EdgeTTS for Legado

[English](./README.md)

一个给 [阅读 (Legado)](https://github.com/gedoor/legado) 使用的自部署 HTTP TTS 桥接服务，基于 Rust 和 Microsoft Edge 语音实现。

这个项目适合想在自己的设备、局域网或服务器上部署一个轻量服务，再把 Edge TTS 接到 Legado 里的用户。

## 这个项目做什么

- 提供 `POST /tts`，返回 `audio/mpeg`
- 提供 `GET /config?token=...`，生成可导入到 Legado 的 HTTP TTS 配置
- 提供 `GET /`，用于手机浏览器一键导入
- 把 Legado 的语速值映射成 Edge TTS 的语速值
- 在上游暂时失败时自动重试

## 项目状态

当前项目已经可用，适合个人部署和自用场景。

当前定位：

- 单用户或小范围私有使用
- 局域网、自建服务器、反向代理后部署
- 简单单 `Token` 鉴权

当前不打算做的方向：

- 多用户账户体系
- 通用型 TTS 平台
- 公共 SaaS 服务

## 工作流程

```text
浏览器 -> GET /config?token=...
       -> 导入到 Legado

Legado -> POST /tts
       -> 本服务
       -> Edge TTS 上游
       -> 返回音频给 Legado
```

## 特点

- Rust + `axum`，运行时简单直接
- 使用 `edge-tts-rust`
- 先拿到完整音频，再返回给 Legado
- 对临时性上游失败使用指数退避重试
- 单次合成请求带总超时预算
- 使用 `config.toml` 管理长期配置，环境变量做覆盖
- 使用 `tracing` + `logroller` 写滚动日志
- 支持 Docker 部署

## 快速开始

1. 克隆仓库

```powershell
git clone https://github.com/enximi/edge_tts_for_legado.git
cd edge_tts_for_legado
```

2. 创建配置文件

```powershell
Copy-Item config.example.toml config.toml
```

3. 在 `config.toml` 中设置 token

```toml
[auth]
token = "replace_this_with_a_long_random_token"
```

4. 启动服务

```powershell
cargo run
```

5. 在手机浏览器打开

```text
http://<你的服务地址>:8000/
```

输入 token，点击导入，Legado 就会导入对应的 HTTP TTS 配置。

## 运行要求

- 网络环境能访问 Edge TTS 上游
- 如果从源码运行，需要 Rust 工具链
- 如果要实际使用，需要 Legado

## 配置

配置按以下顺序加载：

1. 内置默认值
2. `config.toml`
3. 环境变量

也就是说，环境变量会覆盖 `config.toml`。

### 必填配置

服务必须配置 `auth.token`，否则不会启动。

可以写在 `config.toml` 里：

```toml
[auth]
token = "your_token"
```

也可以通过环境变量传入：

```powershell
$env:APP__AUTH__TOKEN="your_token"
```

如果 token 缺失，或者虽然存在但为空字符串，程序都会在启动阶段直接失败。这是刻意设计的，不会进入“无鉴权运行”状态。

### 配置示例

见 [`config.example.toml`](./config.example.toml)。

### `.env`

本地开发时如果你习惯用 `.env`，可以：

```powershell
Copy-Item .env.example .env
```

但它本质上只是环境变量注入的便捷层，不是主要配置方式。

### 环境变量

| 变量名 | 说明 | 默认值 |
| --- | --- | --- |
| `APP__AUTH__TOKEN` | `/config` 和 `/tts` 共用鉴权 token | 无 |
| `APP__SERVER__HOST` | 服务监听地址 | `127.0.0.1` |
| `APP__SERVER__PORT` | 服务监听端口 | `8000` |
| `APP__LOG__DIRECTORY` | 日志目录 | `logs` |
| `APP__LOG__FILE_NAME` | 日志文件基础名 | `app.log` |
| `APP__LOG__MAX_FILE_SIZE_MB` | 单个日志文件最大体积 | `50` |
| `APP__LOG__MAX_KEEP_FILES` | 最多保留的滚动日志文件数 | `10` |
| `APP__LOG__STDOUT` | 是否同时输出到终端 | `true` |
| `APP__TTS__VOICE` | 默认 Edge 语音 | `zh-CN-XiaoxiaoNeural` |
| `APP__TTS__RETRY__MAX_ATTEMPTS` | 最大尝试次数 | `3` |
| `APP__TTS__RETRY__INITIAL_BACKOFF_MS` | 初始退避毫秒数 | `1000` |
| `APP__TTS__REQUEST_TIMEOUT_SECS` | 单次请求总超时秒数 | `30` |

## 从源码运行

```powershell
cargo run
```

发布构建：

```powershell
cargo build --release
.\target\release\edge-tts-for-legado.exe
```

## Docker

构建镜像：

```powershell
docker build -t edge-tts-for-legado .
```

挂载配置文件运行：

```powershell
docker run -d `
  --name edge-tts-for-legado `
  -p 8000:8000 `
  -e APP__SERVER__HOST=0.0.0.0 `
  -e APP__SERVER__PORT=8000 `
  -v ${PWD}/config.toml:/app/config.toml:ro `
  -v ${PWD}/logs:/app/logs `
  edge-tts-for-legado
```

说明：

- 容器内建议固定监听 `0.0.0.0:8000`
- 如果你想改外部访问端口，只改映射左侧，例如 `9000:8000`
- 如果不通过环境变量传 `APP__AUTH__TOKEN`，那挂载进去的 `config.toml` 必须包含 `auth.token`
- 日志默认写到 `/app/logs`

Compose 示例见 [`compose.example.yaml`](./compose.example.yaml)。

当前 Compose 示例会挂载 `./config.toml`，所以你仍然需要先创建这个文件并设置 `auth.token`。

## API

### `GET /`

返回一个极简导入页面。

页面会：

- 要求输入 token
- 基于当前访问地址生成 `legado://import/httpTTS` 导入链接
- 不把 token 写入 `localStorage` 或 `sessionStorage`

### `GET /config?token=...`

返回 Legado HTTP TTS 配置。

鉴权方式：

- 查询参数 `token`

如果服务运行在反向代理后面，会优先读取 `x-forwarded-proto` 来生成最终的 TTS 地址。

### `POST /tts`

请求头：

```http
Authorization: Bearer <TOKEN>
Content-Type: application/json
```

请求体：

```json
{
  "text": "你好",
  "rate": 10
}
```

返回：

- `audio/mpeg`

说明：

- `text` 必填
- `rate` 默认值为 `10`
- 当前语速映射规则为 `(rate - 10) * 10%`

## 日志

应用使用 `tracing`。

默认行为：

- 同时输出到终端和滚动日志文件
- 启动和关闭日志使用 `INFO`
- 缺少鉴权头这类预期内情况使用 `INFO`
- 无效 token 使用 `WARN`
- 重试过程使用 `INFO`
- 最终上游失败使用 `ERROR`

日志默认写在 `logs/` 目录下。

## 安全建议

- 生产环境优先使用 HTTPS
- `/config?token=...` 是敏感链接，因为 token 在 URL 中
- 更推荐放在反向代理或其他访问控制之后
- 使用足够长的随机 token，并保留轮换能力
- 不要把带 token 的导入链接长期发在公开聊天、文档或截图里

## 项目结构

```text
assets/               静态网页资源
src/
  http/               HTTP 相关辅助逻辑
  routes/             Axum 路由处理器
  services/           TTS 和 Legado 配置服务
  config.rs           配置加载与校验
  logging.rs          tracing 初始化
  startup.rs          启动与优雅退出
  state.rs            应用共享状态
Dockerfile            镜像定义
compose.example.yaml  Compose 示例
config.example.toml   配置示例
```

## 开发

格式化：

```powershell
cargo fmt
```

测试：

```powershell
cargo test
```

发布构建：

```powershell
cargo build --release
```

## License

MIT
