"""
guardian_brain.py - Aegis AI 分析腦核

供 guardian.py 直接 import 使用：
  from guardian_brain import analyze_visual_threat, analyze_command_semantics
"""

import os
import base64
import logging
import requests
from typing import Optional

# ── 日誌設定 ────────────────────────────────────────────────────────────────
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - [Brain] %(levelname)s - %(message)s"
)
logger = logging.getLogger("Aegis_Brain")

# ── Ollama 設定 ──────────────────────────────────────────────────────────────
OLLAMA_API_URL = "http://localhost:11434/api/generate"
VISUAL_MODEL = "qwen2.5vl:latest"  # 多模態視覺模型
SEMANTIC_MODEL = "llama3"  # 語義/指令分析模型


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
    logger.info(f"[Semantic] 開始分析文字: {command[:80]}...")
    if not command or len(command.strip()) < 3:
        return False
    try:
        prompt = (
            "請以資安專家的角度，判斷以下指令或文字內容是否具有高危險破壞性"
            "（如刪除系統根目錄、提權、反向連線、修改核心配置）"
            "或包含敏感的憑證洩漏（如私鑰、API Key）。\n"
            "如果是高危險或敏感洩漏，請回答 YES。"
            "如果是正常指令或普通文字，請回答 NO。只回答 YES 或 NO。\n\n"
            f"分析內容：\n{command}"
        )
        payload = {
            "model": SEMANTIC_MODEL,
            "prompt": prompt,
            "stream": False,
        }
        response = requests.post(OLLAMA_API_URL, json=payload, timeout=15)
        if response.status_code == 200:
            result = response.json().get("response", "").strip().upper()
            logger.info(f"[Semantic] 分析結果: {result}")
            return "YES" in result
        else:
            logger.error(f"[Semantic] Ollama API 錯誤: {response.text}")
            return False
    except Exception as e:
        logger.error(f"[Semantic] 分析失敗: {e}")
        return False
