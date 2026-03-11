import time
import pyperclip
import re
import logging
import threading
from core.models.incident import IncidentLogger

logger = logging.getLogger('Aegis_Guardian')

# 需由 main.py 初始化時注入
i18n = None 

class ClipboardMonitor(threading.Thread):
    """
    (Controller)
    監控剪貼簿內容，並根據 config 中的 regex 或 AI 語義分析進行比對。
    """
    def __init__(self, config, notifier, ai_client=None, mitigator=None):
        super().__init__()
        self.config = config
        self.notifier = notifier
        self.rules = config.get('behavior_firewall', {}).get('regex_rules', {})
        self.ai_client = ai_client
        self.mitigator = mitigator
        self.daemon = True
        self.last_content = ""
        self.running = True

    def run(self):
        logger.info(i18n.get('clipboard_monitor_started') if i18n else "Clipboard Monitor Started")
        while self.running:
            try:
                current_content = pyperclip.paste()
                if current_content != self.last_content:
                    self.check_content(current_content)
                    self.last_content = current_content
            except Exception as e:
                logger.error(i18n.get('clipboard_monitor_error', error=e) if i18n else f"Clipboard error: {e}")
            
            time.sleep(1)

    def check_content(self, content):
        if not content:
            return

        regex_triggered = False
        for rule_name, pattern in self.rules.items():
            if re.search(pattern, content):
                regex_triggered = True
                msg = i18n.get('clipboard_alert', rule_name=rule_name) if i18n else f"Regex triggered: {rule_name}"
                logger.warning(msg)
                
                IncidentLogger.record(
                    module="ClipboardMonitor",
                    severity="WARNING",
                    message=f"Sensitive data detected (matched rule: {rule_name})",
                    metadata={"rule_name": rule_name, "preview": content[:100]}
                )

                if self.mitigator:
                    self.mitigator.auto_mitigate_incident(
                        module="ClipboardMonitor",
                        severity="WARNING",
                        metadata={"rule_name": rule_name, "preview": content[:100]}
                    )

                try:
                    pyperclip.copy("[REDACTED BY AEGIS]")
                    logger.info("Clipboard cleared for safety.")
                    if i18n:
                        msg += "\n" + i18n.get('clipboard_redacted')
                except Exception as e:
                    logger.error(f"Clipboard clear failed: {e}")
                
                self.notifier.send_alert(msg)
                break  

        if not regex_triggered and self.ai_client and self.ai_client.available:
            logger.info("🧠 [AI Brain] 剪貼簿未命中 regex，啟動 AI 語義分析...")
            is_threat = self.ai_client.analyze_semantic(content)
            if is_threat:
                ai_msg = (
                    "🤖 [AI 判定] 剪貼簿內容觸發語義威脅警報："
                    "偵測到高危險指令或敏感憑證。"
                )
                logger.warning(ai_msg)
                IncidentLogger.record(
                    module="AI_Brain_Clipboard",
                    severity="CRITICAL",
                    message="AI 語義分析判定剪貼簿內容具高危險性",
                    metadata={"preview": content[:200], "model": "llama3"}
                )
                try:
                    pyperclip.copy("[AI REDACTED BY AEGIS]")
                    logger.info("[AI Brain] 剪貼簿已由 AI 清空。")
                except Exception as e:
                    logger.error(f"[AI Brain] 清空剪貼簿失敗: {e}")
                self.notifier.send_alert(ai_msg)

    def stop(self):
        self.running = False
