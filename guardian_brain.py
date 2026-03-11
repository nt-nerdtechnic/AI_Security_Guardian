import requests
import base64
import json
import logging
from typing import Dict, Any

# 設定日誌
logging.basicConfig(level=logging.INFO, format='%(asctime)s - [Brain] %(levelname)s - %(message)s')
logger = logging.getLogger('Aegis_Brain')

OLLAMA_API_URL = "http://localhost:11434/api/generate"
# 預設使用的本地模型
VISUAL_MODEL = "qwen2.5-vl"
SEMANTIC_MODEL = "llama3" # 或其他您本地有的輕量級 LLM

def analyze_visual_threat(image_path: str) -> bool:
    """
    接收圖片路徑，呼叫本地多模態大模型判斷是否有資安風險。
    """
    logger.info(f"開始分析視覺威脅: {image_path}")
    try:
        with open(image_path, "rb") as image_file:
            encoded_string = base64.b64encode(image_file.read()).decode('utf-8')
            
        prompt = "這是一張電腦螢幕截圖。請判斷畫面中是否出現了系統權限請求視窗（如 sudo 密碼輸入框、TCC 權限請求）或明確的密碼明文。如果有的話回答 YES，沒有的話回答 NO。只回答 YES 或 NO。"
        
        payload = {
            "model": VISUAL_MODEL,
            "prompt": prompt,
            "images": [encoded_string],
            "stream": False
        }
        
        response = requests.post(OLLAMA_API_URL, json=payload, timeout=30)
        
        if response.status_code == 200:
            result = response.json().get('response', '').strip().upper()
            logger.info(f"視覺分析結果: {result}")
            return "YES" in result
        else:
            logger.error(f"Ollama API 錯誤: {response.text}")
            return False
            
    except Exception as e:
        logger.error(f"視覺威脅分析失敗: {e}")
        return False

def analyze_command_semantics(command: str) -> bool:
    """
    分析終端機指令或剪貼簿文字的語義意圖，判斷是否為高危險操作。
    """
    logger.info(f"開始分析語義威脅: {command}")
    if not command or len(command.strip()) < 3:
        return False
        
    try:
        prompt = f"""請以資安專家的角度，判斷以下指令或文字內容是否具有高危險破壞性（如刪除系統根目錄、提權、反向連線、修改核心配置）或包含敏感的憑證洩漏（如私鑰、API Key）。
如果是高危險或敏感洩漏，請回答 YES。如果是正常指令或普通文字，請回答 NO。只回答 YES 或 NO。

分析內容：
{command}"""
        
        payload = {
            "model": SEMANTIC_MODEL,
            "prompt": prompt,
            "stream": False
        }
        
        response = requests.post(OLLAMA_API_URL, json=payload, timeout=15)
        
        if response.status_code == 200:
            result = response.json().get('response', '').strip().upper()
            logger.info(f"語義分析結果: {result}")
            return "YES" in result
        else:
            logger.error(f"Ollama API 錯誤: {response.text}")
            return False
            
    except Exception as e:
        logger.error(f"語義威脅分析失敗: {e}")
        return False

if __name__ == "__main__":
    # 簡單的本地測試
    print("Aegis Brain Server is ready.")
