import edge_tts
import json
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse, StreamingResponse
from fastapi.templating import Jinja2Templates
from pydantic import BaseModel

app = FastAPI()
templates = Jinja2Templates(directory="templates")


class TTSRequest(BaseModel):
    text: str
    rate: int = 15


@app.post("/tts")
async def tts_endpoint(request: TTSRequest):
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
    """
    Return the Legado configuration.
    Dynamically replaces the host in the URL with the request's host.
    """
    host = request.headers.get("host") or "localhost:8000"

    # Construct the URL for the TTS endpoint
    # Note: Legado requires the URL to be http://...
    # The body template strictly follows Legado's requirement
    tts_url = f"http://{host}/tts"

    # Define the JSON body pattern Legado should send
    # We inline everything for compactness as requested
    config = {
        "concurrentRate": "1000",
        "contentType": "audio/mpeg",
        "header": json.dumps({"Content-Type": "application/json"}),
        "id": 1735914000000,
        "loginCheckJs": "",
        "loginUi": "",
        "loginUrl": "",
        "name": "EdgeTTS for Legado",
        "url": f"{tts_url},{json.dumps({'method': 'POST', 'body': json.dumps({'text': '{{speakText}}', 'rate': '{{speakSpeed}}'})})}",
    }

    return JSONResponse(content=config)


@app.get("/")
async def index(request: Request):
    """
    Web page with one-click import button.
    """
    return templates.TemplateResponse(request=request, name="index.html")
