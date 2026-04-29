"""
guardian_brain.py - Aegis AI 分析腦核

供 guardian.py 直接 import 使用：
  from guardian_brain import analyze_visual_threat, analyze_command_semantics
"""

import os
import base64
import json
import logging
import requests
import yaml
from pathlib import Path

# ── 日誌設定 ────────────────────────────────────────────────────────────────
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - [Brain] %(levelname)s - %(message)s"
)
logger = logging.getLogger("Aegis_Brain")

# ── Ollama 設定 ──────────────────────────────────────────────────────────────
OLLAMA_API_URL = "http://localhost:11434/api/generate"
DEFAULT_VISUAL_MODEL = "qwen2.5vl:latest"
DEFAULT_SEMANTIC_MODEL = "llama3"
VISUAL_MODEL = os.getenv("AEGIS_VISUAL_MODEL", DEFAULT_VISUAL_MODEL)
SEMANTIC_MODEL = os.getenv("AEGIS_SEMANTIC_MODEL", DEFAULT_SEMANTIC_MODEL)


def _config_path() -> Path:
    if os.getenv("AEGIS_CONFIG_PATH"):
        return Path(os.environ["AEGIS_CONFIG_PATH"])
    return Path(__file__).resolve().parent / "config.yaml"


def load_ai_settings() -> dict:
    settings = {
        "ollama_url": os.getenv("AEGIS_OLLAMA_URL", OLLAMA_API_URL),
        "semantic_model": os.getenv("AEGIS_SEMANTIC_MODEL", DEFAULT_SEMANTIC_MODEL),
        "visual_model": os.getenv("AEGIS_VISUAL_MODEL", DEFAULT_VISUAL_MODEL),
        "rule_packs": [],
    }
    try:
        config = yaml.safe_load(_config_path().read_text(encoding="utf-8")) or {}
        ai_cfg = config.get("ai", {})
        model_cfg = ai_cfg.get("models", {})
        settings["ollama_url"] = ai_cfg.get("ollama_url", settings["ollama_url"])
        settings["semantic_model"] = model_cfg.get(
            "semantic", settings["semantic_model"]
        )
        settings["visual_model"] = model_cfg.get("visual", settings["visual_model"])
        settings["rule_packs"] = ai_cfg.get("rule_packs", [])
    except Exception as exc:
        logger.debug(f"[AI Config] 使用預設模型設定: {exc}")
    return settings


def refresh_ai_settings() -> dict:
    global OLLAMA_API_URL, SEMANTIC_MODEL, VISUAL_MODEL
    settings = load_ai_settings()
    OLLAMA_API_URL = settings["ollama_url"]
    SEMANTIC_MODEL = settings["semantic_model"]
    VISUAL_MODEL = settings["visual_model"]
    return settings


def _empty_analysis(reason: str = "") -> dict:
    return {
        "verdict": "safe",
        "confidence": 0.0,
        "category": "unknown",
        "reason": reason,
        "recommended_action": "monitor",
    }


def parse_semantic_analysis(raw_response: str) -> dict:
    try:
        data = json.loads(raw_response.strip())
    except json.JSONDecodeError:
        verdict = "threat" if "YES" in raw_response.upper() else "safe"
        return {
            "verdict": verdict,
            "confidence": 0.5 if verdict == "threat" else 0.2,
            "category": "legacy_text",
            "reason": raw_response.strip()[:300],
            "recommended_action": "redact" if verdict == "threat" else "monitor",
        }
    if not isinstance(data, dict):
        return _empty_analysis("model response was not a JSON object")

    verdict = str(data.get("verdict", "safe")).lower()
    if verdict not in {"safe", "suspicious", "threat"}:
        verdict = "safe"
    try:
        confidence = max(0.0, min(1.0, float(data.get("confidence", 0.0))))
    except (TypeError, ValueError):
        confidence = 0.0

    return {
        "verdict": verdict,
        "confidence": confidence,
        "category": str(data.get("category", "unknown")),
        "reason": str(data.get("reason", ""))[:500],
        "recommended_action": str(data.get("recommended_action", "monitor")),
    }


refresh_ai_settings()


# ════════════════════════════════════════════════════════════════════════════
# 核心分析函數
# ════════════════════════════════════════════════════════════════════════════


def analyze_visual_threat(image_path: str) -> bool:
    """
    接收圖片路徑，呼叫本地多模態大模型判斷是否有資安風險。
    回傳 True 表示偵測到威脅。
    """
    logger.info(f"[Visual] 開始分析截圖: {image_path}")
    try:
        if not os.path.exists(image_path):
            logger.warning(f"[Visual] 截圖檔案不存在: {image_path}")
            return False

        with open(image_path, "rb") as f:
            encoded = base64.b64encode(f.read()).decode("utf-8")

        prompt = (
            "這是一張電腦螢幕截圖。請判斷畫面中是否出現了系統權限請求視窗"
            "（如 sudo 密碼輸入框、TCC 權限請求）或明確的密碼明文。"
            "如果有的話回答 YES，沒有的話回答 NO。只回答 YES 或 NO。"
        )
        payload = {
            "model": VISUAL_MODEL,
            "prompt": prompt,
            "images": [encoded],
            "stream": False,
        }
        response = requests.post(OLLAMA_API_URL, json=payload, timeout=30)
        if response.status_code == 200:
            result = response.json().get("response", "").strip().upper()
            logger.info(f"[Visual] 分析結果: {result}")
            return "YES" in result
        else:
            logger.error(f"[Visual] Ollama API 錯誤: {response.text}")
            return False
    except Exception as e:
        logger.error(f"[Visual] 分析失敗: {e}")
        return False


def analyze_command_semantics(command: str) -> bool:
    """
    分析終端機指令或剪貼簿文字的語義意圖，判斷是否為高危險操作。
    回傳 True 表示偵測到威脅。
    """
    return analyze_command_semantics_detailed(command)["verdict"] in {
        "suspicious",
        "threat",
    }


def analyze_command_semantics_detailed(command: str) -> dict:
    """
    分析終端機指令或剪貼簿文字的語義意圖，回傳穩定 JSON schema：
    verdict, confidence, category, reason, recommended_action。
    """
    logger.info(f"[Semantic] 開始分析文字: {command[:80]}...")
    if not command or len(command.strip()) < 3:
        return _empty_analysis("input too short")
    try:
        prompt = (
            "請以資安專家的角度，判斷以下指令或文字內容是否具有高危險破壞性"
            "（如刪除系統根目錄、提權、反向連線、修改核心配置）"
            "或包含敏感的憑證洩漏（如私鑰、API Key）。\n"
            "請只輸出 JSON，schema 固定如下："
            '{"verdict":"safe|suspicious|threat","confidence":0.0,'
            '"category":"credential|destructive_command|network|privilege|benign|unknown",'
            '"reason":"short reason","recommended_action":"monitor|redact|block|quarantine"}。\n\n'
            f"分析內容：\n{command}"
        )
        payload = {
            "model": SEMANTIC_MODEL,
            "prompt": prompt,
            "stream": False,
        }
        response = requests.post(OLLAMA_API_URL, json=payload, timeout=15)
        if response.status_code == 200:
            result = response.json().get("response", "").strip()
            analysis = parse_semantic_analysis(result)
            logger.info(f"[Semantic] 分析結果: {analysis}")
            return analysis
        else:
            logger.error(f"[Semantic] Ollama API 錯誤: {response.text}")
            return _empty_analysis("ollama api error")
    except Exception as e:
        logger.error(f"[Semantic] 分析失敗: {e}")
        return _empty_analysis(str(e))
