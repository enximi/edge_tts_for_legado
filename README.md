# EdgeTTS for Legado

[简体中文](./README.zh-CN.md)

A self-hosted HTTP TTS bridge for [Legado](https://github.com/gedoor/legado), powered by Rust and Microsoft Edge voices.

This project is for people who want to use Edge TTS inside Legado through a small service they control themselves.

## What It Does

- Exposes a `POST /tts` endpoint that returns `audio/mpeg`
- Exposes a `GET /config?token=...` endpoint that generates a Legado HTTP TTS config
- Exposes a minimal `GET /` import page for mobile browsers
- Translates Legado speech rate values to Edge TTS rate values
- Retries temporary upstream synthesis failures

## Project Status

This project is usable today for personal deployment.

Current scope:

- single-user or small private deployments
- local network, reverse proxy, or self-hosted server setups
- simple token-based authentication

Non-goals for now:

- multi-user account systems
- a general-purpose TTS platform
- a public SaaS-style deployment model

## How It Works

```text
Browser -> GET /config?token=...
        -> import into Legado

Legado -> POST /tts
       -> this service
       -> Edge TTS upstream
       -> audio/mpeg back to Legado
```

## Features

- Rust + `axum` service with a small runtime footprint
- `edge-tts-rust` integration
- full-audio response mode instead of proxy-streaming partial audio
- exponential backoff retry for transient upstream failures
- total timeout budget per synthesis request
- configuration from `config.toml` with environment-variable overrides
- rotating log files using `tracing` and `logroller`
- Docker support for Linux deployment

## Quick Start

1. Clone the repository.

```powershell
git clone https://github.com/enximi/edge_tts_for_legado.git
cd edge_tts_for_legado
```

2. Create a config file.

```powershell
Copy-Item config.example.toml config.toml
```

3. Set a token in `config.toml`.

```toml
[auth]
token = "replace_this_with_a_long_random_token"
```

4. Start the service.

```powershell
cargo run
```

5. Open the import page in your phone browser.

```text
http://<your-server>:8000/
```

Enter the token, tap import, and Legado will import the generated HTTP TTS profile.

## Requirements

- a network environment that can reach the Edge TTS upstream
- Rust if you want to run from source
- Legado if you want to use the generated config

## Configuration

Configuration is loaded in this order:

1. built-in defaults
2. `config.toml`
3. environment variables

That means environment variables override `config.toml`.

### Required Setting

The service will not start unless `auth.token` is configured.

You can set it either in `config.toml`:

```toml
[auth]
token = "your_token"
```

or through an environment variable:

```powershell
$env:APP__AUTH__TOKEN="your_token"
```

If the token is missing or blank, startup fails by design.

### Example Config

See [`config.example.toml`](./config.example.toml).

### Optional `.env`

For local development, you can also use `.env`:

```powershell
Copy-Item .env.example .env
```

This is just a convenience layer for environment variables. It is not the primary configuration format.

### Environment Variables

| Variable | Description | Default |
| --- | --- | --- |
| `APP__AUTH__TOKEN` | Shared auth token for `/config` and `/tts` | none |
| `APP__SERVER__HOST` | Bind host | `127.0.0.1` |
| `APP__SERVER__PORT` | Bind port | `8000` |
| `APP__LOG__DIRECTORY` | Log directory | `logs` |
| `APP__LOG__FILE_NAME` | Base log file name | `app.log` |
| `APP__LOG__MAX_FILE_SIZE_MB` | Max size per log file | `50` |
| `APP__LOG__MAX_KEEP_FILES` | Max rotated log files to keep | `10` |
| `APP__LOG__STDOUT` | Also write logs to stdout | `true` |
| `APP__TTS__VOICE` | Default Edge voice | `zh-CN-XiaoxiaoNeural` |
| `APP__TTS__RETRY__MAX_ATTEMPTS` | Max synthesis attempts | `3` |
| `APP__TTS__RETRY__INITIAL_BACKOFF_MS` | Initial retry backoff in ms | `1000` |
| `APP__TTS__REQUEST_TIMEOUT_SECS` | Total timeout budget per request | `30` |

## Running From Source

```powershell
cargo run
```

Release build:

```powershell
cargo build --release
.\target\release\edge-tts-for-legado.exe
```

## Docker

Build the image:

```powershell
docker build -t edge-tts-for-legado .
```

Run it with a mounted config file:

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

Notes:

- keep the container bound to `0.0.0.0:8000`
- change the external port through port mapping, for example `9000:8000`
- `config.toml` must include `auth.token` unless you provide `APP__AUTH__TOKEN`
- logs are written to `/app/logs`

Compose example: [`compose.example.yaml`](./compose.example.yaml)

The sample Compose file mounts `./config.toml`, so you still need to create that file and set `auth.token`.

## API

### `GET /`

Returns a minimal import page.

The page:

- asks for the token
- generates a `legado://import/httpTTS` link for the current origin
- does not store the token in `localStorage` or `sessionStorage`

### `GET /config?token=...`

Returns a Legado HTTP TTS configuration payload.

Authentication:

- query parameter `token`

If the service is behind a reverse proxy, `x-forwarded-proto` is used when building the final TTS URL.

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

Response:

- `audio/mpeg`

Notes:

- `text` is required
- `rate` defaults to `10`
- Legado rate is mapped as `(rate - 10) * 10%`

## Logging

The service uses `tracing`.

Default behavior:

- logs are written to stdout and to rotating files
- successful startup and shutdown are `INFO`
- missing auth headers are `INFO`
- invalid tokens are `WARN`
- retry attempts are `INFO`
- final upstream failures are `ERROR`

By default, logs are written under `logs/`.

## Security Notes

- use HTTPS in production when possible
- treat `/config?token=...` as sensitive because the token is in the URL
- prefer running behind a reverse proxy or another access-control layer
- use a long random token and rotate it when needed
- avoid posting import URLs in public chats or screenshots

## Project Layout

```text
assets/               Static web assets
src/
  http/               HTTP-specific helpers
  routes/             Axum handlers
  services/           TTS and Legado config services
  config.rs           Config loading and validation
  logging.rs          Tracing setup
  startup.rs          Bootstrap and shutdown
  state.rs            Shared application state
Dockerfile            Container image definition
compose.example.yaml  Docker Compose example
config.example.toml   Config example
```

## Development

Format:

```powershell
cargo fmt
```

Test:

```powershell
cargo test
```

Build release:

```powershell
cargo build --release
```

## License

MIT
