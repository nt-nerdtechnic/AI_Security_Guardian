import time
import subprocess
import logging
import threading
from datetime import datetime
from pathlib import Path
from core.models.incident import IncidentLogger

logger = logging.getLogger('Aegis_Guardian')
BASE_DIR = Path(__file__).resolve().parent.parent.parent
LOGS_DIR = BASE_DIR / 'logs'

i18n = None 

def cleanup_old_snapshots(max_keep: int = 50):
    try:
        patterns = ['snapshot_*.png', 'terminal_alert_*.png', 'sprint_verify_*.png']
        all_snapshots = []
        for pattern in patterns:
            all_snapshots.extend(LOGS_DIR.glob(pattern))

        if len(all_snapshots) <= max_keep:
            return

        all_snapshots.sort(key=lambda p: p.stat().st_mtime)
        to_delete = all_snapshots[:len(all_snapshots) - max_keep]
        for f in to_delete:
            try:
                f.unlink()
            except Exception:
                pass
        logger.info(f"[清理] 刪除 {len(to_delete)} 張舊截圖，保留最新 {max_keep} 張。")
    except Exception as e:
        logger.error(f"[清理] 快照清理失敗: {e}")

class ActiveWindowMonitor(threading.Thread):
    def __init__(self, config, notifier, ai_client=None):
        super().__init__()
        self.config = config
        self.notifier = notifier
        self.ai_client = ai_client
        self.target_windows = config.get('visual_sentry', {}).get('target_windows', [])
        self.daemon = True
        self.running = True
        self.last_app = ""
        self.last_snapshot_time = 0
        self.cooldown = 10 

    def get_frontmost_info(self):
        try:
            script = 'tell application "System Events" to tell (first application process whose frontmost is true) to get {name, name of window 1}'
            result = subprocess.check_output(['osascript', '-e', script], timeout=2)
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
            return

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filepath = LOGS_DIR / f"snapshot_{trigger_name}_{timestamp}.png"
        try:
            subprocess.run(['screencapture', '-x', str(filepath)], check=True)
            self.last_snapshot_time = current_time
            
            msg = i18n.get('visual_alert', trigger_name=trigger_name, filename=filepath.name) if i18n else f"Visual alert: {trigger_name}"
            logger.warning(msg)

            IncidentLogger.record(
                module="VisualSentry",
                severity="WARNING",
                message=f"Privilege window detected: {trigger_name}",
                metadata={"trigger": trigger_name, "snapshot_file": str(filepath)}
            )

            self.notifier.send_snapshot(filepath, msg)

            max_keep = self.config.get('visual_sentry', {}).get('max_snapshots', 50)
            cleanup_old_snapshots(max_keep)

            if self.ai_client and self.ai_client.available:
                logger.info(f"🧠 [AI Brain] 對截圖進行視覺威脅分析: {filepath.name}")
                is_visual_threat = self.ai_client.analyze_visual(str(filepath))
                if is_visual_threat:
                    ai_msg = f"🤖 [AI 視覺判定] 截圖 {filepath.name} 偵測到特權提權畫面或密碼外洩風險。"
                    logger.warning(ai_msg)
                    IncidentLogger.record(
                        module="AI_Brain_Visual",
                        severity="CRITICAL",
                        message="AI 視覺分析判定截圖具高危險性",
                        metadata={"trigger": trigger_name, "snapshot_file": str(filepath)}
                    )
                    self.notifier.send_alert(ai_msg)
        except Exception as e:
            logger.error(f"Snapshot failed: {e}")

    def run(self):
        logger.info(i18n.get('visual_monitor_started') if i18n else "Visual monitor started")
        while self.running:
            try:
                app_name, window_title = self.get_frontmost_info()
                if app_name != self.last_app:
                    if app_name:
                        for target in self.target_windows:
                            if target.lower() in app_name.lower() or target.lower() in window_title.lower():
                                self.take_snapshot(target)
                                break
                    self.last_app = app_name
            except Exception as e:
                pass
            time.sleep(2.0)

    def stop(self):
        self.running = False
