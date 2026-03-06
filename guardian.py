import os
import time
import yaml
import logging
import re
import threading
import json
import pyperclip
import requests
import subprocess
import psutil
from datetime import datetime
from pathlib import Path
from pynput import keyboard
from guardian_core.actions.mitigation import kill_process_by_pid
from guardian_core.i18n import I18nManager

# 建立全域的 I18nManager 實例（預設 zh-TW，稍後在 main 中依設定變更）
i18n = I18nManager('zh-TW')

# ============================================================================
# Core Configuration
# ============================================================================
BASE_DIR = Path(__file__).resolve().parent
CONFIG_FILE = BASE_DIR / 'config.yaml'
LOGS_DIR = BASE_DIR / 'logs'
INCIDENTS_JSON = LOGS_DIR / 'incidents.json'

# Logger Setup
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
    ]
)
logger = logging.getLogger('Aegis_Guardian')

def ensure_environment():
    """確保必要的目錄（如 logs/）存在"""
    if not LOGS_DIR.exists():
        LOGS_DIR.mkdir(parents=True, exist_ok=True)
        # 移除舊的未配置好的 FileHandler，替換為新的
        logger.handlers = [h for h in logger.handlers if not isinstance(h, logging.FileHandler)]
        logger.addHandler(logging.FileHandler(LOGS_DIR / 'guardian.log', mode='a'))
        logger.info(i18n.get('log_dir_created', path=LOGS_DIR))
    else:
        logger.info(i18n.get('log_dir_exists', path=LOGS_DIR))

def record_incident(module, severity, message, metadata=None):
    """
    Records a security incident in a structured JSON format.
    Appends the incident to logs/incidents.json as Newline Delimited JSON (NDJSON).
    """
    incident = {
        "timestamp": datetime.now().isoformat(),
        "module": module,
        "severity": severity,
        "message": message,
        "metadata": metadata or {}
    }
    
    try:
        with open(INCIDENTS_JSON, "a", encoding="utf-8") as f:
            f.write(json.dumps(incident, ensure_ascii=False) + "\n")
    except Exception as e:
        logger.error(i18n.get('record_incident_failed', error=e))

def load_config():
    """載入 config.yaml 設定檔"""
    if not CONFIG_FILE.exists():
        logger.error(i18n.get('config_not_found'))
        return None

    try:
        with open(CONFIG_FILE, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
            logger.info(i18n.get('config_loaded'))
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
                logger.debug(i18n.get('telegram_alert_sent'))
            else:
                logger.error(i18n.get('telegram_alert_failed', error=resp.text))
        except Exception as e:
            logger.error(i18n.get('telegram_alert_failed', error=e))

    def send_snapshot(self, filepath, caption=""):
        """傳送圖片檔案到指定 Telegram"""
        if not self.bot_token or not self.chat_id:
            logger.debug("Telegram credentials not configured. Skipping snapshot upload.")
            return

        url = f"https://api.telegram.org/bot{self.bot_token}/sendPhoto"
        try:
            with open(filepath, 'rb') as f:
                files = {'photo': f}
                data = {'chat_id': self.chat_id, 'caption': caption}
                resp = requests.post(url, data=data, files=files, timeout=30)
                if resp.status_code == 200:
                    logger.debug(i18n.get('telegram_snapshot_sent'))
                else:
                    logger.error(i18n.get('telegram_snapshot_failed', error=resp.text))
        except Exception as e:
            logger.error(i18n.get('telegram_snapshot_failed', error=e))


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
        logger.info(i18n.get('clipboard_monitor_started'))
        while self.running:
            try:
                current_content = pyperclip.paste()
                if current_content != self.last_content:
                    self.check_content(current_content)
                    self.last_content = current_content
            except Exception as e:
                logger.error(i18n.get('clipboard_monitor_error', error=e))
            
            time.sleep(1)

    def check_content(self, content):
        if not content:
            return

        for rule_name, pattern in self.rules.items():
            if re.search(pattern, content):
                msg = i18n.get('clipboard_alert', rule_name=rule_name)
                logger.warning(msg)
                print(f"\n{msg}")
                
                # Record incident to JSON
                record_incident(
                    module="ClipboardMonitor",
                    severity="WARNING",
                    message=f"Sensitive data detected (matched rule: {rule_name})",
                    metadata={"rule_name": rule_name, "preview": content[:100]}
                )

                # Proactive action: Clear clipboard to prevent accidental leak
                try:
                    pyperclip.copy("[REDACTED BY GUARDIAN]")
                    logger.info("Clipboard cleared for safety.")
                    msg += "\n" + i18n.get('clipboard_redacted')
                except Exception as e:
                    logger.error(i18n.get('clipboard_clear_failed', error=e))
                
                self.notifier.send_alert(msg)

    def stop(self):
        self.running = False


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
        self.last_app = ""
        self.last_snapshot_time = 0
        self.cooldown = 10  # Seconds cooldown between snapshots

    def get_frontmost_info(self):
        """獲取作用中應用程式名稱與視窗標題"""
        try:
            # 透過 AppleScript 查詢當前最高層級的應用程式名稱與視窗標題
            script = 'tell application "System Events" to tell (first application process whose frontmost is true) to get {name, name of window 1}'
            result = subprocess.check_output(['osascript', '-e', script], timeout=2)
            # 解析結果，例如 "Terminal, Logs"
            output = result.decode('utf-8').strip()
            parts = [p.strip() for p in output.split(',')]
            app_name = parts[0] if len(parts) > 0 else ""
            window_title = parts[1] if len(parts) > 1 else ""
            return app_name, window_title
        except Exception:
            return "", ""

    def take_snapshot(self, trigger_name):
        current_time = time.time()
        if current_time - self.last_snapshot_time < self.cooldown:
            logger.debug(i18n.get('visual_snapshot_cooldown', trigger_name=trigger_name))
            return

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filepath = LOGS_DIR / f"snapshot_{trigger_name}_{timestamp}.png"
        try:
            # 使用系統自帶 screencapture (-x 靜音)
            subprocess.run(['screencapture', '-x', str(filepath)], check=True)
            self.last_snapshot_time = current_time
            
            msg = i18n.get('visual_alert', trigger_name=trigger_name, filename=filepath.name)
            logger.warning(msg)
            print(f"\n{msg}")

            # Record incident to JSON
            record_incident(
                module="VisualSentry",
                severity="WARNING",
                message=f"Privilege window detected: {trigger_name}",
                metadata={"trigger": trigger_name, "snapshot_file": str(filepath)}
            )

            # Auto-send file via Telegram if configured
            self.notifier.send_snapshot(filepath, msg)
        except Exception as e:
            logger.error(i18n.get('visual_snapshot_failed', error=e))

    def run(self):
        logger.info(i18n.get('visual_monitor_started'))
        while self.running:
            try:
                app_name, window_title = self.get_frontmost_info()
                
                # Check if app changed or if it's a target app
                if app_name != self.last_app:
                    if app_name:
                        for target in self.target_windows:
                            if target.lower() in app_name.lower() or target.lower() in window_title.lower():
                                self.take_snapshot(target)
                                break
                    self.last_app = app_name
                # If app is same, we don't re-trigger snapshot to avoid title-change spam
                    
            except Exception as e:
                logger.error(i18n.get('visual_monitor_error', error=e))
            time.sleep(2.0)

    def stop(self):
        self.running = False


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
        self.running = True
        
    def get_frontmost_app(self):
        try:
            script = 'tell application "System Events" to get name of first application process whose frontmost is true'
            result = subprocess.check_output(['osascript', '-e', script])
            return result.decode('utf-8').strip()
        except:
            return ""

    def on_press(self, key):
        if not self.running:
            return False
            
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
                        msg = i18n.get('terminal_alert', command=cmd, keyword=kw)
                        logger.warning(msg)
                        print(f"\n{msg}")

                        # Record incident to JSON
                        record_incident(
                            module="TerminalFirewall",
                            severity="CRITICAL",
                            message=f"High risk command detected",
                            metadata={"command": cmd, "matched_keyword": kw}
                        )
                        
                        # Phase R&D: Take visual proof of the high-risk command in terminal
                        active_app = self.get_frontmost_app()
                        if active_app in ['Terminal', 'iTerm2']:
                            # Using a local helper or direct call to ActiveWindowMonitor logic
                            # For simplicity in this incremental step, we trigger a dedicated capture
                            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                            filepath = LOGS_DIR / f"terminal_alert_{timestamp}.png"
                            try:
                                subprocess.run(['screencapture', '-x', str(filepath)], check=True)
                                self.notifier.send_snapshot(filepath, f"🚨 {msg}")
                            except Exception as e:
                                logger.error(i18n.get('terminal_capture_failed', error=e))

                        self.notifier.send_alert(msg)
        except Exception as e:
            pass

    def run(self):
        logger.info(i18n.get('terminal_monitor_started'))
        with keyboard.Listener(on_press=self.on_press) as self.listener:
            while self.running:
                time.sleep(1)
            self.listener.stop()

    def stop(self):
        self.running = False


class SystemHeartbeat(threading.Thread):
    """
    系統健康檢查心跳，每 1 分鐘紀錄一次運行狀態。
    """
    def __init__(self, monitors):
        super().__init__()
        self.monitors = monitors
        self.daemon = True
        self.running = True

    def run(self):
        logger.info(i18n.get('heartbeat_started'))
        process = psutil.Process(os.getpid())
        while self.running:
            active_modules = [m.__class__.__name__ for m in self.monitors if m.is_alive()]
            
            # Resource monitoring
            mem_info = process.memory_info()
            cpu_usage = process.cpu_percent(interval=None)
            mem_mb = mem_info.rss / 1024 / 1024
            
            status_msg = i18n.get('heartbeat_status', module_count=len(active_modules), cpu=cpu_usage, mem=round(mem_mb, 1))
            logger.info(status_msg)
            
            # Auto-restart/Critical alert logic if thresholds exceeded
            # Configurable limits for R&D stability
            MAX_MEM_MB = 200 
            MAX_CPU_PERCENT = 80

            if mem_mb > MAX_MEM_MB:
                msg = i18n.get('mem_leak_shutdown', mem=round(mem_mb, 1))
                logger.critical(msg)
                os._exit(1) # Immediate exit to trigger potential OS-level restart or alert

            if cpu_usage > MAX_CPU_PERCENT:
                logger.warning(i18n.get('resource_cpu_high', cpu=cpu_usage))
                
            time.sleep(60) # Increased frequency for R&D phase

    def stop(self):
        self.running = False


class NetworkMonitor(threading.Thread):
    """
    監控網路連線，識別異常流量或可疑連接埠。
    """
    def __init__(self, config, notifier):
        super().__init__()
        self.config = config.get('network_monitor', {})
        self.notifier = notifier
        self.daemon = True
        self.running = True
        self.interval = self.config.get('check_interval', 5)
        self.threshold = self.config.get('high_bandwidth_threshold_mb', 50) * 1024 * 1024
        self.suspicious_ports = self.config.get('suspicious_ports', [])
        self.target_processes = self.config.get('target_processes', [])
        self.last_stats = {}

    def run(self):
        logger.info(i18n.get('network_monitor_started'))
        while self.running:
            try:
                # 使用 lsof 獲取當前連線，避免 psutil.net_connections 權限問題
                # -i: 選擇網路檔案, -n: 不解析主機名, -P: 不解析連接埠名
                output = subprocess.check_output(['lsof', '-i', '-nP'], stderr=subprocess.STDOUT).decode()
                lines = output.splitlines()
                if len(lines) > 1:
                    headers = lines[0].split()
                    for line in lines[1:]:
                        parts = line.split()
                        if len(parts) < 9: continue
                        
                        command = parts[0]
                        pid = parts[1]
                        # 檢查遠端地址 (通常在第 9 欄位)
                        # 格式如: 192.168.1.1:1234->1.2.3.4:4444 (ESTABLISHED)
                        remote_info = parts[8]
                        if '->' in remote_info:
                            remote_addr = remote_info.split('->')[1]
                            if ':' in remote_addr:
                                port_str = remote_addr.split(':')[-1]
                                try:
                                    port = int(port_str)
                                    if port in self.suspicious_ports:
                                        msg = i18n.get('network_alert', port=port, command=command, pid=pid)
                                        logger.warning(msg)
                                        record_incident(
                                            module="NetworkMonitor",
                                            severity="WARNING",
                                            message="Suspicious outbound port detected",
                                            metadata={"port": port, "pid": pid, "command": command}
                                        )
                                        self.notifier.send_alert(msg)
                                        
                                        # Incremental Step: Auto-mitigation for suspicious ports
                                        # Only kill if the process name isn't a known safe one (to avoid killing the agent itself or dev tools)
                                        # Added 'Code' (VSCode) and 'Antigravity' to exclusion list to prevent workspace disruption
                                        if command not in ['Python', 'node', 'ssh', 'iterm2', 'terminal', 'Code', 'Antigravity']:
                                            if kill_process_by_pid(pid):
                                                mitigation_msg = i18n.get('mitigation_killed', command=command, pid=pid, port=port)
                                                logger.warning(mitigation_msg)
                                                self.notifier.send_alert(mitigation_msg)
                                except:
                                    continue
            except Exception as e:
                logger.error(i18n.get('network_monitor_error', error=e))
            
            time.sleep(self.interval)

    def get_proc_name(self, pid):
        try:
            return psutil.Process(pid).name()
        except:
            return "Unknown"

    def stop(self):
        self.running = False


# ============================================================================
# Main Event Loop
# ============================================================================
def main():
    logger.info("Starting Aegis Guardian...") # Will be overwritten by i18n after config load, but keep it initial if config missing
    ensure_environment()
    
    config = load_config()
    if not config:
        return
        
    global i18n
    i18n.load_language(config.get('language', 'zh-TW'))
    logger.info(i18n.get('system_starting'))

    # 初始化通報器
    notifier = TelegramNotifier(config)

    # 初始化監控模組
    monitors = [
        ClipboardMonitor(config, notifier),
        ActiveWindowMonitor(config, notifier),
        KeystrokeMonitor(config, notifier),
        NetworkMonitor(config, notifier)
    ]

    # 啟動監控模組
    for m in monitors:
        m.start()

    # 啟動系統心跳
    heartbeat = SystemHeartbeat(monitors)
    heartbeat.start()

    # R&D Increment: Periodic Snapshot for verification
    def periodic_status_capture():
        while True:
            time.sleep(3600) # Hourly verify
            timestamp = datetime.now().strftime("%Y%m%d_%H%M")
            filepath = LOGS_DIR / f"sprint_verify_{timestamp}.png"
            try:
                subprocess.run(['screencapture', '-x', str(filepath)], check=True)
                logger.info(f"Verification snapshot saved: {filepath.name}")
            except:
                pass

    threading.Thread(target=periodic_status_capture, daemon=True).start()

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
