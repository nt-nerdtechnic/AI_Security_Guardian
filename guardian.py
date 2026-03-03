import os
import time
import yaml
import logging
import re
import threading
import pyperclip
import requests
import subprocess
from datetime import datetime
from pathlib import Path
from pynput import keyboard

# ============================================================================
# Core Configuration
# ============================================================================
BASE_DIR = Path(__file__).resolve().parent
CONFIG_FILE = BASE_DIR / 'config.yaml'
LOGS_DIR = BASE_DIR / 'logs'

# Logger Setup
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler(LOGS_DIR / 'guardian.log', mode='a') if LOGS_DIR.exists() else logging.StreamHandler()
    ]
)
logger = logging.getLogger('AI_Guardian')

def ensure_environment():
    """確保必要的目錄（如 logs/）存在"""
    if not LOGS_DIR.exists():
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        # 移除舊的未配置好的 FileHandler，替換為新的
        logger.handlers = [h for h in logger.handlers if not isinstance(h, logging.FileHandler)]
        logger.addHandler(logging.FileHandler(LOGS_DIR / 'guardian.log', mode='a'))
        logger.info(f"Directory created: {LOGS_DIR}")
    else:
        logger.info(f"Directory exists: {LOGS_DIR}")

def load_config():
    """載入 config.yaml 設定檔"""
    if not CONFIG_FILE.exists():
        logger.error(f"Configuration file not found: {CONFIG_FILE}")
        return None

    try:
        with open(CONFIG_FILE, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
            logger.info("Configuration loaded successfully.")
            return config
    except Exception as e:
        logger.error(f"Failed to load configuration: {e}")
        return None


# ============================================================================
# Webhook Notifier (B)
# ============================================================================
class TelegramNotifier:
    def __init__(self, config):
        self.config = config.get('webhook', {}).get('telegram', {})
        self.bot_token = self.config.get('bot_token', '')
        self.chat_id = self.config.get('chat_id', '')

    def send_alert(self, message):
        """傳送文字訊息到指定 Telegram"""
        if not self.bot_token or not self.chat_id:
            logger.debug("Telegram credentials not configured. Skipping webhook.")
            return

        url = f"https://api.telegram.org/bot{self.bot_token}/sendMessage"
        payload = {"chat_id": self.chat_id, "text": message}
        try:
            resp = requests.post(url, json=payload, timeout=5)
            if resp.status_code == 200:
                logger.debug("Telegram alert sent successfully.")
            else:
                logger.error(f"Failed to send Telegram alert: {resp.text}")
        except Exception as e:
            logger.error(f"Telegram webhook exception: {e}")


# ============================================================================
# Modules
# ============================================================================

class ClipboardMonitor(threading.Thread):
    """
    監控剪貼簿內容，並根據 config 中的 regex 進行比對。
    """
    def __init__(self, config, notifier):
        super().__init__()
        self.config = config
        self.notifier = notifier
        self.rules = config.get('behavior_firewall', {}).get('regex_rules', {})
        self.daemon = True
        self.last_content = ""
        self.running = True

    def run(self):
        logger.info("Clipboard Monitor started.")
        while self.running:
            try:
                current_content = pyperclip.paste()
                if current_content != self.last_content:
                    self.check_content(current_content)
                    self.last_content = current_content
            except Exception as e:
                logger.error(f"Error in Clipboard Monitor: {e}")
            
            time.sleep(1)

    def check_content(self, content):
        if not content:
            return

        for rule_name, pattern in self.rules.items():
            if re.search(pattern, content):
                msg = f"⚠️ [SECURITY ALERT] Sensitive data detected in clipboard ({rule_name})!"
                logger.warning(msg)
                print(f"\n{msg}")
                self.notifier.send_alert(msg)

    def stop(self):
        self.running = False
        if hasattr(self, "listener"):
            self.listener.stop()


class ActiveWindowMonitor(threading.Thread):
    """
    監聽 MacOS 當前的作用中視窗 (Frontmost Application)。
    一旦偵測到特權相關視窗，啟動截圖存證。
    """
    def __init__(self, config, notifier):
        super().__init__()
        self.config = config
        self.notifier = notifier
        # 視覺哨兵目標白名單
        self.target_windows = config.get('visual_sentry', {}).get('target_windows', [])
        self.daemon = True
        self.running = True
        self.last_window = ""

    def get_frontmost_info(self):
        """獲取作用中應用程式名稱與視窗標題"""
        try:
            # 透過 AppleScript 查詢當前最高層級的應用程式名稱與視窗標題
            script = 'tell application "System Events" to tell (first application process whose frontmost is true) to get {name, name of window 1}'
            result = subprocess.check_output(['osascript', '-e', script])
            # 解析結果，例如 "Terminal, Logs"
            parts = result.decode('utf-8').strip().split(', ')
            app_name = parts[0] if len(parts) > 0 else ""
            window_title = parts[1] if len(parts) > 1 else ""
            return app_name, window_title
        except Exception:
            return "", ""

    def take_snapshot(self, trigger_name):
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filepath = LOGS_DIR / f"snapshot_{trigger_name}_{timestamp}.png"
        try:
            # 使用系統自帶 screencapture (-x 靜音)
            subprocess.run(['screencapture', '-x', str(filepath)], check=True)
            msg = f"📸 [VISUAL SENTRY] Privilege window detected ({trigger_name}). Snapshot saved: {filepath.name}"
            logger.warning(msg)
            print(f"\n{msg}")
            self.notifier.send_alert(msg)
        except Exception as e:
            logger.error(f"Failed to take snapshot: {e}")

    def run(self):
        logger.info("Active Window Monitor (Visual Sentry) started.")
        while self.running:
            try:
                app_name, window_title = self.get_frontmost_info()
                combined_info = f"{app_name} | {window_title}"
                
                if combined_info != self.last_window:
                    # 檢查是否為特權或目標關注視窗 (App 名稱或 視窗標題)
                    for target in self.target_windows:
                        if target.lower() in app_name.lower() or target.lower() in window_title.lower():
                            self.take_snapshot(target)
                            break
                    self.last_window = combined_info
            except Exception as e:
                logger.error(f"Error in Active Window Monitor: {e}")
            time.sleep(1.5)

    def stop(self):
        self.running = False
        if hasattr(self, "listener"):
            self.listener.stop()


class KeystrokeMonitor(threading.Thread):
    """
    終端指令預審 (Terminal Rules / Firewall)。
    藉由 pynput 掛鉤記錄鍵盤敲擊。只有在終端機或 iTerm2 是 Focus 的狀態下記錄 Buffer。
    按下 Enter 後判斷命令是否帶有高風險意圖（如 rm -rf /）。
    """
    def __init__(self, config, notifier):
        super().__init__()
        self.config = config
        self.notifier = notifier
        self.high_risk_keywords = config.get('terminal_rules', {}).get('high_risk_keywords', [])
        self.daemon = True
        self.buffer = []
        
    def get_frontmost_app(self):
        try:
            script = 'tell application "System Events" to get name of first application process whose frontmost is true'
            result = subprocess.check_output(['osascript', '-e', script])
            return result.decode('utf-8').strip()
        except:
            return ""

    def on_press(self, key):
        active_app = self.get_frontmost_app()
        # 限定只在 Terminal 或 iTerm2 活躍時紀錄 (可降低資源消耗與隱私疑慮)
        if active_app not in ['Terminal', 'iTerm2']:
            return

        try:
            if hasattr(key, 'char') and key.char is not None:
                self.buffer.append(key.char)
            elif key == keyboard.Key.space:
                self.buffer.append(' ')
            elif key == keyboard.Key.backspace and self.buffer:
                self.buffer.pop()
            elif key == keyboard.Key.enter:
                # 組裝字串並清空 buffer
                cmd = "".join(self.buffer).strip()
                self.buffer.clear()
                if not cmd:
                    return
                # 簡單字串比對：高風險詞彙檢查
                for kw in self.high_risk_keywords:
                    if kw in cmd:
                        msg = f"🛑 [TERMINAL FIREWALL] High risk command detected: '{cmd}' (matched '{kw}')"
                        logger.warning(msg)
                        print(f"\n{msg}")
                        self.notifier.send_alert(msg)
        except Exception as e:
            pass

    def run(self):
        logger.info("Keystroke Monitor (Terminal Rules) started.")
        self.running = True
        self.buffer = []
        with keyboard.Listener(on_press=self.on_press) as self.listener:
            while self.running:
                time.sleep(1)
            self.listener.stop()

    def on_press(self, key):
        if not self.running:
            return False
        active_app = self.get_frontmost_app()

    def stop(self):
        self.running = False
        if hasattr(self, "listener"):
            self.listener.stop()


# ============================================================================
# Main Event Loop
# ============================================================================
def main():
    logger.info("Starting AI Security Guardian...")
    ensure_environment()
    
    config = load_config()
    if not config:
        logger.critical("Cannot proceed without configuration. Exiting.")
        return

    # 初始化通報器
    notifier = TelegramNotifier(config)

    # 模組1: 剪貼簿防火牆
    clipboard_monitor = ClipboardMonitor(config, notifier)
    clipboard_monitor.start()

    # 模組2: 視覺哨兵 (截圖擷取)
    window_monitor = ActiveWindowMonitor(config, notifier)
    window_monitor.start()

    # 模組3: 終端機按鍵側錄攔截
    keystroke_monitor = KeystrokeMonitor(config, notifier)
    keystroke_monitor.start()

    logger.info("AI Security Guardian is now monitoring the system. (Modules initialized)")
    
    try:
        while True:
            time.sleep(1)
            
    except KeyboardInterrupt:
        logger.info("Shutting down AI Security Guardian gracefully...")
        clipboard_monitor.stop()
        window_monitor.stop()
        keystroke_monitor.stop()
    except Exception as e:
        logger.error(f"Unexpected error: {e}", exc_info=True)


if __name__ == '__main__':
    main()
