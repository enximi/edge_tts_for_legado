import os

import edge_tts
from dotenv import load_dotenv
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse, StreamingResponse
from fastapi.templating import Jinja2Templates
from pydantic import BaseModel

load_dotenv()
API_KEY = os.getenv("API_KEY")
if not API_KEY:
    raise ValueError("API_KEY environment variable is required but not set.")

app = FastAPI()
templates = Jinja2Templates(directory="templates")


class TTSRequest(BaseModel):
    text: str
    rate: int = 15


async def verify_query_api_key(request: Request) -> bool:
    # Check query param (for config url mainly)
    api_key_query = request.query_params.get("api_key")
    if api_key_query == API_KEY:
        return True
    return False


async def verify_header_api_key(request: Request) -> bool:
    # Check header
    auth_header = request.headers.get("Authorization")
    if auth_header:
        scheme, _, param = auth_header.partition(" ")
        if scheme.lower() == "bearer" and param == API_KEY:
            return True
    return False


@app.post("/tts", dependencies=[])
async def tts_endpoint(request: TTSRequest, req: Request):
    if not await verify_header_api_key(req):
        raise HTTPException(status_code=401, detail="Invalid API Key")

    """
    Stream audio from edge-tts.
    Legado passes rate as int 5-50, default 15.
    We convert this to % for edge-tts.
    """
    if not request.text:
        raise HTTPException(status_code=400, detail="Text is empty")

    # Calculate rate percentage
    # 15 -> 0%
    # Each unit is 5%
    rate_diff = request.rate - 15
    rate_pct = rate_diff * 5
    rate_str = f"{rate_pct:+d}%"

    communicate = edge_tts.Communicate(
        text=request.text, voice="zh-CN-XiaoxiaoNeural", rate=rate_str, volume="+0%"
    )

    # edge-tts stream returns chunks of bytes
    async def audio_stream():
        async for chunk in communicate.stream():
            if chunk["type"] == "audio":
                yield chunk["data"]

    return StreamingResponse(audio_stream(), media_type="audio/mpeg")


@app.get("/config")
async def get_config(request: Request):
    if not await verify_query_api_key(request):
        raise HTTPException(status_code=401, detail="Invalid API Key")

    """
    Return the Legado configuration.
    Dynamically replaces the host in the URL with the request's host.
    """
    host = request.headers.get("host") or "localhost:8000"

    # Construct the URL for the TTS endpoint
    tts_url = f"http://{host}/tts"

    # Legado config format: url,{"method": "POST", "body": "..."}
    # We manually construct the strings to avoid double-escaping issues.

    # Body: {{speakSpeed}} is an int, so no quotes. Inner quotes must be escaped.
    body_str = '{\\"text\\": \\"{{speakText}}\\", \\"rate\\": {{speakSpeed}}}'

    # Request config
    req_config = f'{{"method": "POST", "body": "{body_str}"}}'

    # Add Authorization header to the Legado config
    header_str = f'{{\n"Content-Type": "application/json",\n"Authorization": "Bearer {API_KEY}"\n}}'

    config = {
        "concurrentRate": "1000",
        "contentType": "audio/mpeg",
        "header": header_str,
        "id": 1735914000000,
        "loginCheckJs": "",
        "loginUi": "",
        "loginUrl": "",
        "name": "EdgeTTS for Legado",
        "url": f"{tts_url},{req_config}",
    }

    return JSONResponse(content=config)


@app.get("/")
async def index(request: Request):
    """
    Web page with one-click import button.
    """
    return templates.TemplateResponse(request=request, name="index.html")
