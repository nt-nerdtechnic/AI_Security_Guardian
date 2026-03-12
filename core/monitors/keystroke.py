import time
import subprocess
import logging
import threading
from datetime import datetime
from pynput import keyboard
from pathlib import Path
from core.models.incident import IncidentLogger

logger = logging.getLogger("Aegis_Guardian")
BASE_DIR = Path(__file__).resolve().parent.parent.parent
LOGS_DIR = BASE_DIR / "logs"

i18n = None


class KeystrokeMonitor(threading.Thread):
    def __init__(self, config, notifier):
        super().__init__()
        self.config = config
        self.notifier = notifier
        self.high_risk_keywords = config.get("terminal_rules", {}).get(
            "high_risk_keywords", []
        )
        self.daemon = True
        self.buffer = []
        self.running = True

    def get_frontmost_app(self):
        try:
            script = 'tell application "System Events" to get name of first application process whose frontmost is true'
            result = subprocess.check_output(["osascript", "-e", script])
            return result.decode("utf-8").strip()
        except:
            return ""

    def on_press(self, key):
        if not self.running:
            return False

        active_app = self.get_frontmost_app()
        if active_app not in ["Terminal", "iTerm2"]:
            return

        try:
            if hasattr(key, "char") and key.char is not None:
                self.buffer.append(key.char)
            elif key == keyboard.Key.space:
                self.buffer.append(" ")
            elif key == keyboard.Key.backspace and self.buffer:
                self.buffer.pop()
            elif key == keyboard.Key.enter:
                cmd = "".join(self.buffer).strip()
                self.buffer.clear()
                if not cmd:
                    return
                for kw in self.high_risk_keywords:
                    if kw in cmd:
                        msg = (
                            i18n.get("terminal_alert", command=cmd, keyword=kw)
                            if i18n
                            else f"Regex triggered: {kw}"
                        )
                        logger.warning(msg)

                        IncidentLogger.record(
                            module="TerminalFirewall",
                            severity="CRITICAL",
                            message="High risk command detected",
                            metadata={"command": cmd, "matched_keyword": kw},
                        )

                        active_app = self.get_frontmost_app()
                        if active_app in ["Terminal", "iTerm2"]:
                            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
                            filepath = LOGS_DIR / f"terminal_alert_{timestamp}.png"
                            try:
                                subprocess.run(
                                    ["screencapture", "-x", str(filepath)], check=True
                                )
                                self.notifier.send_snapshot(filepath, f"🚨 {msg}")
                            except Exception as e:
                                logger.error(f"Capture failed: {e}")

                        self.notifier.send_alert(msg)
        except Exception:
            pass

    def run(self):
        logger.info(
            i18n.get("terminal_monitor_started") if i18n else "Terminal Monitor started"
        )
        with keyboard.Listener(on_press=self.on_press) as self.listener:
            while self.running:
                time.sleep(1)
            self.listener.stop()

    def stop(self):
        self.running = False
