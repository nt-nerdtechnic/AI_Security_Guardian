import sys
import logging
import time
import requests
import psutil
from pathlib import Path

# 初始化根目錄，確保可以載入 guardian_core 和 guardian_brain
BASE_DIR = Path(__file__).resolve().parent.parent
sys.path.append(str(BASE_DIR))

from core.models.config import ConfigModel
from core.models.incident import IncidentLogger
from core.viewmodels.notifier import TelegramNotifierViewModel
from core.viewmodels.ai_client import AiBrainViewModel
from core.monitors.clipboard import ClipboardMonitor
from core.monitors.active_window import ActiveWindowMonitor
from core.monitors.keystroke import KeystrokeMonitor
from core.monitors.network import NetworkMonitor
from core.monitors.heartbeat import SystemHeartbeat

from core.models.i18n import I18nManager
import core.monitors.clipboard as cb_mon
import core.monitors.active_window as aw_mon
import core.monitors.keystroke as k_mon
import core.monitors.network as nw_mon
import core.monitors.heartbeat as hb_mon

# Logger Setup
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[logging.StreamHandler()]
)
logger = logging.getLogger('Aegis_Guardian')

def handle_telegram_callback(callback_query):
    """處理來自 Telegram Inline 案件的回傳資料"""
    data = callback_query.get('data', '')
    msg_id = callback_query.get('message', {}).get('message_id')
    chat_id = callback_query.get('message', {}).get('chat_id')
    bot_token = callback_query.get('bot_token')
    
    if '|' not in data:
        return

    action, target_info = data.split('|', 1)
    response_text = ""
    
    if action == "quarantine":
        source_path = Path(target_info.split('->')[0] if '->' in target_info else target_info)
        response_text = f"✅ [遙控] 請求隔離檔案：{source_path}"
            
    elif action == "terminate":
        try:
            pid = int(target_info)
            p = psutil.Process(pid)
            name = p.name()
            p.terminate()
            response_text = f"✅ [成功] 已終止進程：{name} (PID: {pid})"
        except Exception as e:
            response_text = f"❌ [失敗] 無法終止進程：{e}"
            
    elif action == "ignore":
        response_text = f"🙈 [已忽略] 警告：{target_info}"

    url = f"https://api.telegram.org/bot{bot_token}/editMessageText"
    payload = {
        "chat_id": chat_id,
        "message_id": msg_id,
        "text": f"{callback_query.get('message', {}).get('text')}\n\n---\n{response_text}"
    }
    requests.post(url, json=payload)

def main():
    logger.info("Starting Aegis Guardian (MVVM Architecture)...")
    IncidentLogger.ensure_log_dir()
    
    # [Model] 載入設定
    config = ConfigModel.load()
    if not config:
        return

    # [i18n] 注入語系檔至各模組
    i18n = I18nManager(config.get('language', 'zh-TW'))
    cb_mon.i18n = i18n
    aw_mon.i18n = i18n
    k_mon.i18n = i18n
    nw_mon.i18n = i18n
    hb_mon.i18n = i18n
    
    logger.info(i18n.get('system_starting'))

    # [ViewModel] 初始化服務
    notifier = TelegramNotifierViewModel(config)
    notifier.start_polling(handle_telegram_callback)
    
    ai_client = AiBrainViewModel()

    # [Controllers] 初始化並啟動監控
    monitors = [
        ClipboardMonitor(config, notifier, ai_client=ai_client),
        ActiveWindowMonitor(config, notifier, ai_client=ai_client),
        KeystrokeMonitor(config, notifier),
        NetworkMonitor(config, notifier)
    ]

    for m in monitors:
        m.start()

    heartbeat = SystemHeartbeat(monitors)
    heartbeat.start()

    logger.info(i18n.get('system_starting') + " (Modules initialized)")
    
    try:
        while True:
            time.sleep(1)
            
    except KeyboardInterrupt:
        logger.info(i18n.get('shutdown_gracefully'))
        for m in monitors:
            m.stop()
        heartbeat.stop()
    except Exception as e:
        logger.error(i18n.get('unexpected_error', error=e), exc_info=True)

if __name__ == '__main__':
    main()
