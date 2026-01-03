# Edge TTS for Legado

这是一个为 [阅读 (Legado)](https://github.com/gedoor/legado) 设计的 Microsoft Edge TTS 代理服务。它基于 [edge-tts](https://github.com/rany2/edge-tts) Python 库，可以将高品质的流式语音集成到阅读 App 中。

## 功能特点

- **高质量语音**：基于 `edge-tts` 库调用 Microsoft Edge TTS API，支持 `zh-CN-XiaoxiaoNeural`（晓晓）等优秀音效。
- **集成导入**：内置 Web 界面，支持一键将配置导入到 Legado。
- **API 安全**：支持通过 `API_KEY` 进行简单的身份验证。

## 快速开始

### 前置要求

- Python 3.12+
- [uv](https://github.com/astral-sh/uv) (推荐的 Python 包管理工具)

### 运行步骤

1. **克隆项目**：

   ```bash
   git clone <repository-url>
   cd edge_tts_for_legado
   ```

2. **配置环境变量**：
   创建 `.env` 文件并设置你的 `API_KEY`：

   ```bash
   echo "API_KEY=your_secret_key" > .env
   ```

3. **安装依赖并启动**：

   ```bash
   uv run uvicorn main:app --host 0.0.0.0 --port 8000
   ```

## 配置说明

### 环境变量

| 变量名    | 说明                           | 示例                |
| :-------- | :----------------------------- | :------------------ |
| `API_KEY` | 用于身份验证的密钥。必须设置。 | `my_secure_api_key` |

### 在 Legado 中配置

1. **一键导入（推荐）**：
   在手机浏览器访问 `http://<your-server-ip>:8000/`，点击页面上的“导入到阅读”按钮（需在手机端操作，且已安装 Legado）。

2. **手动导入**：
   将以下 URL 添加到 Legado 的语音引擎配置中：
   `http://<your-server-ip>:8000/config?api_key=your_secret_key`

## 许可证

MIT License
