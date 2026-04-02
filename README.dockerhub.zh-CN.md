# EdgeTTS for Legado

一个给 [阅读 (Legado)](https://github.com/gedoor/legado) 使用的自部署 HTTP TTS 桥接镜像，基于 Rust 和 Microsoft Edge 语音实现。

GitHub 仓库：<https://github.com/enximi/edge_tts_for_legado>

## 这个镜像做什么

这个镜像会运行一个小型 HTTP 服务，用来：

- 提供 `POST /tts`，返回 `audio/mpeg`
- 提供 `GET /config?token=...`，生成可导入到 Legado 的 HTTP TTS 配置
- 提供 `GET /`，作为手机浏览器的一键导入页面

它适合个人部署、家庭服务器、局域网环境，以及放在反向代理后面的私有使用场景。

## 重要说明

这个服务必须配置 token。

如果 `config.toml` 和环境变量里都没有提供 `auth.token`，容器会启动失败。

## 快速开始

### 1. 创建配置文件

创建 `config.toml`：

```toml
[server]
host = "0.0.0.0"
port = 8000

[auth]
token = "replace_this_with_a_long_random_token"

[log]
directory = "logs"
file_name = "app.log"
max_file_size_mb = 50
max_keep_files = 10
stdout = true

[tts]
voice = "zh-CN-XiaoxiaoNeural"
request_timeout_secs = 30

[tts.retry]
max_attempts = 3
initial_backoff_ms = 1000
```

### 2. 运行容器

```bash
docker run -d \
  --name edge-tts-for-legado \
  -p 8000:8000 \
  -e APP__SERVER__HOST=0.0.0.0 \
  -e APP__SERVER__PORT=8000 \
  -v $(pwd)/config.toml:/app/config.toml:ro \
  -v $(pwd)/logs:/app/logs \
  enximi/edge-tts-for-legado:latest
```

然后在浏览器打开：

```text
http://<你的服务地址>:8000/
```

输入 token 后，把生成的配置导入到 Legado 即可。

## Docker Compose 示例

```yaml
services:
  edge-tts-for-legado:
    image: enximi/edge-tts-for-legado:latest
    restart: unless-stopped
    ports:
      - "8000:8000"
    environment:
      APP__SERVER__HOST: 0.0.0.0
      APP__SERVER__PORT: 8000
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./logs:/app/logs
```

## 配置优先级

配置按以下顺序加载：

1. 内置默认值
2. `config.toml`
3. 环境变量

环境变量会覆盖 `config.toml`。

## 环境变量

- `APP__AUTH__TOKEN`
- `APP__SERVER__HOST`
- `APP__SERVER__PORT`
- `APP__LOG__DIRECTORY`
- `APP__LOG__FILE_NAME`
- `APP__LOG__MAX_FILE_SIZE_MB`
- `APP__LOG__MAX_KEEP_FILES`
- `APP__LOG__STDOUT`
- `APP__TTS__VOICE`
- `APP__TTS__RETRY__MAX_ATTEMPTS`
- `APP__TTS__RETRY__INITIAL_BACKOFF_MS`
- `APP__TTS__REQUEST_TIMEOUT_SECS`

## API

### `GET /`

手机浏览器使用的极简导入页面。

### `GET /config?token=...`

返回 Legado HTTP TTS 配置。

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

## 安全建议

- 使用足够长的随机 token
- 生产环境优先使用 HTTPS
- `/config?token=...` 属于敏感链接
- 更推荐放在反向代理或其他访问控制之后

## 源码

GitHub：<https://github.com/enximi/edge_tts_for_legado>
