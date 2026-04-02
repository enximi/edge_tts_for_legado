# EdgeTTS for Legado

A self-hosted HTTP TTS bridge for [Legado](https://github.com/gedoor/legado), powered by Rust and Microsoft Edge voices.

GitHub repository: <https://github.com/enximi/edge_tts_for_legado>

## What This Image Does

This image runs a small HTTP service that:

- exposes `POST /tts` and returns `audio/mpeg`
- exposes `GET /config?token=...` for generating a Legado HTTP TTS config
- exposes `GET /` as a minimal mobile import page

It is designed for personal deployment, home servers, and private reverse-proxy setups.

## Important

This service requires a token.

If neither `config.toml` nor environment variables provide `auth.token`, the container will fail to start.

## Quick Start

### 1. Create a config file

Create `config.toml`:

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

### 2. Run the container

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

Then open:

```text
http://<your-server>:8000/
```

Enter your token and import the generated config into Legado.

## Docker Compose Example

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

## Configuration Priority

Configuration is loaded in this order:

1. built-in defaults
2. `config.toml`
3. environment variables

Environment variables override `config.toml`.

## Environment Variables

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

Minimal import page for mobile browsers.

### `GET /config?token=...`

Returns a Legado HTTP TTS configuration payload.

### `POST /tts`

Headers:

```http
Authorization: Bearer <TOKEN>
Content-Type: application/json
```

Body:

```json
{
  "text": "Hello",
  "rate": 10
}
```

Returns:

- `audio/mpeg`

## Security Notes

- Use a long random token
- Use HTTPS in production
- Treat `/config?token=...` as sensitive
- Prefer running behind a reverse proxy or another access-control layer

## Source

GitHub: <https://github.com/enximi/edge_tts_for_legado>
