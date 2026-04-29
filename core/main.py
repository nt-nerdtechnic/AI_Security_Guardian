import sys
import base64
import hashlib
import hmac as _hmac_mod
import logging
import time
import json
import requests
import psutil
from pathlib import Path

# 初始化根目錄，確保可以載入 guardian_core 和 guardian_brain
BASE_DIR = Path(__file__).resolve().parent.parent
sys.path.append(str(BASE_DIR))

from core.models.config import ConfigModel
from core.models.incident import INCIDENTS_JSON, IncidentLogger
from core.viewmodels.notifier import TelegramNotifierViewModel
from core.viewmodels.ai_client import AiBrainViewModel
from core.viewmodels.mitigator import MitigationManager
from core.monitors.clipboard import ClipboardMonitor
from core.monitors.active_window import ActiveWindowMonitor
from core.monitors.keystroke import KeystrokeMonitor
from core.monitors.network import NetworkMonitor
from core.monitors.heartbeat import SystemHeartbeat
from core.monitors.system_resource import SystemResourceMonitor

from core.models.i18n import I18nManager
import core.monitors.clipboard as cb_mon
import core.monitors.active_window as aw_mon
import core.monitors.keystroke as k_mon
import core.monitors.network as nw_mon
import core.monitors.heartbeat as hb_mon
import core.monitors.system_resource as sr_mon

# Logger Setup
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler()],
)
logger = logging.getLogger("Aegis_Guardian")
CALLBACK_MITIGATOR = None
PROTECTED_PROCESS_NAMES = {
    "kernel_task",
    "launchd",
    "WindowServer",
    "loginwindow",
    "syslogd",
}


def _verify_telegram_callback(callback_query: dict) -> bool:
    """驗證 Telegram callback_query 的來源合法性。

    兩層防護：
    1. 發送者身分驗證：callback_query.from.id 必須符合設定的 chat_id。
    2. HMAC-SHA256 簽名驗證：callback_data 尾端簽名標籤必須符合，
       防止任何人偷改 callback_data 來欺騙系統執行任意操作。
    """
    bot_token: str = callback_query.get("bot_token", "")
    authorized_chat_id: str = str(callback_query.get("authorized_chat_id", ""))
    sender_id: str = str(callback_query.get("from", {}).get("id", ""))

    # ---- 驗證一：發送者身分 ----------------------------------------
    if not authorized_chat_id or sender_id != authorized_chat_id:
        logger.warning(
            "[Security] Telegram callback 來自未授權 sender_id=%s "
            "(期望對象=%s)。已丟棄。",
            sender_id,
            authorized_chat_id,
        )
        return False

    # ---- 驗證二：HMAC-SHA256 簽名 ---------------------------------
    data: str = callback_query.get("data", "")
    parts = data.rsplit("|", 1)
    if len(parts) != 2 or not bot_token:
        logger.warning(
            "[Security] Telegram callback 缺少 HMAC 標籤或 bot_token。已丟棄。"
        )
        return False

    raw_data, received_tag = parts
    expected_sig = _hmac_mod.new(
        bot_token.encode("utf-8"),
        raw_data.encode("utf-8"),
        hashlib.sha256,
    ).digest()[:6]
    expected_tag = base64.urlsafe_b64encode(expected_sig).decode("ascii")

    if not _hmac_mod.compare_digest(expected_tag, received_tag):
        logger.warning(
            "[Security] Telegram callback HMAC 不符——可能遇到偽造攻擊。已丟棄。"
        )
        return False

    return True


def _unsigned_callback_data(callback_query: dict) -> str:
    data = callback_query.get("data", "")
    return data.rsplit("|", 1)[0] if "|" in data else data


def _metadata_path_value(metadata: dict) -> str | None:
    for key in ("file_path", "path", "source", "source_path", "target_path"):
        value = metadata.get(key)
        if isinstance(value, str) and value:
            return value
    return None


def _find_recorded_file_path(target_info: str) -> Path | None:
    """Only allow quarantine for file paths already recorded in incidents.json."""
    target = str(
        Path(target_info.split("->")[0] if "->" in target_info else target_info)
    )
    if not INCIDENTS_JSON.exists():
        return None

    try:
        lines = INCIDENTS_JSON.read_text(encoding="utf-8").splitlines()[-500:]
    except Exception:
        return None

    for line in reversed(lines):
        try:
            incident = json.loads(line)
        except json.JSONDecodeError:
            continue

        path_value = _metadata_path_value(incident.get("metadata", {}))
        if not path_value:
            continue

        candidate = Path(path_value)
        if str(candidate) == target and candidate.exists() and candidate.is_file():
            return candidate
    return None


def _is_protected_pid(pid: int) -> tuple[bool, str]:
    if pid in (0, 1):
        return True, "critical PID"
    if pid == psutil.Process().pid:
        return True, "guardian process"
    if pid == psutil.Process().ppid():
        return True, "guardian parent process"

    try:
        proc = psutil.Process(pid)
        if proc.name() in PROTECTED_PROCESS_NAMES:
            return True, f"protected process name: {proc.name()}"
    except psutil.NoSuchProcess:
        return False, ""
    except Exception as exc:
        return True, f"cannot inspect process safely: {exc}"

    return False, ""


def _terminate_pid_with_audit(pid: int) -> str:
    denied, reason = _is_protected_pid(pid)
    if denied:
        IncidentLogger.record(
            module="TelegramCallback",
            severity="WARNING",
            message="Remote terminate denied",
            metadata={"pid": pid, "reason": reason},
        )
        return f"[Denied] Remote terminate blocked for PID {pid}: {reason}"

    try:
        proc = psutil.Process(pid)
        name = proc.name()
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except psutil.TimeoutExpired:
            proc.kill()
        IncidentLogger.record(
            module="TelegramCallback",
            severity="WARNING",
            message="Remote terminate executed",
            metadata={"pid": pid, "process_name": name},
        )
        return f"[Success] Terminated process: {name} (PID: {pid})"
    except Exception as e:
        IncidentLogger.record(
            module="TelegramCallback",
            severity="ERROR",
            message="Remote terminate failed",
            metadata={"pid": pid, "error": str(e)},
        )
        return f"[Failed] Cannot terminate process: {e}"


def handle_telegram_callback(callback_query):
    """處理來自 Telegram Inline 案件的回嬹資料"""
    # ── 來源驗證：發送者身分 + HMAC-SHA256 簽名 ─────────────────────
    if not _verify_telegram_callback(callback_query):
        return
    # ───────────────────────────────────────────────────────────────────────

    # 剥除 HMAC 簽名標籤（最後一段），還原原始 action|target_info
    data = _unsigned_callback_data(callback_query)

    msg_id = callback_query.get("message", {}).get("message_id")
    chat_id = callback_query.get("message", {}).get("chat", {}).get("id")
    bot_token = callback_query.get("bot_token")

    if "|" not in data:
        return

    action, target_info = data.split("|", 1)
    response_text = ""

    if action == "quarantine":
        source_path = _find_recorded_file_path(target_info)
        if not source_path:
            IncidentLogger.record(
                module="TelegramCallback",
                severity="WARNING",
                message="Remote quarantine denied",
                metadata={
                    "target": target_info,
                    "reason": "path not found in incident metadata",
                },
            )
            response_text = "[Denied] Quarantine target was not found in recorded incident metadata."
        elif CALLBACK_MITIGATOR is None:
            response_text = "[Failed] Mitigation manager is not available."
        else:
            ok = CALLBACK_MITIGATOR.quarantine_file(str(source_path))
            response_text = (
                f"[Success] Quarantined file: {source_path}"
                if ok
                else f"[Failed] Could not quarantine file: {source_path}"
            )

    elif action == "terminate":
        try:
            pid = int(target_info)
            response_text = _terminate_pid_with_audit(pid)
        except Exception as e:
            response_text = f"[Failed] Invalid terminate request: {e}"

    elif action == "ignore":
        response_text = f"[Ignored] Warning: {target_info}"

    url = f"https://api.telegram.org/bot{bot_token}/editMessageText"
    payload = {
        "chat_id": chat_id,
        "message_id": msg_id,
        "text": f"{callback_query.get('message', {}).get('text')}\n\n---\n{response_text}",
    }
    requests.post(url, json=payload)


def main():
    global CALLBACK_MITIGATOR
    logger.info("Starting Aegis Guardian (MVVM Architecture)...")
    IncidentLogger.ensure_log_dir()

    # [Model] 載入設定
    config = ConfigModel.load()
    if not config:
        return

    # [i18n] 注入語系檔至各模組
    i18n = I18nManager(config.get("language", "zh-TW"))
    cb_mon.i18n = i18n
    aw_mon.i18n = i18n
    k_mon.i18n = i18n
    nw_mon.i18n = i18n
    hb_mon.i18n = i18n
    sr_mon.i18n = i18n

    logger.info(i18n.get("system_starting"))

    # [ViewModel] 初始化服務
    notifier = TelegramNotifierViewModel(config)

    ai_client = AiBrainViewModel()
    mitigator = MitigationManager(config)
    CALLBACK_MITIGATOR = mitigator
    notifier.start_polling(handle_telegram_callback)

    # [Controllers] 初始化並啟動監控
    monitors = []

    # 根據 config.yaml 中的 modules 設定決定是否啟動特定模組
    modules_cfg = config.get("modules", {})

    if modules_cfg.get("clipboard", True):
        monitors.append(
            ClipboardMonitor(config, notifier, ai_client=ai_client, mitigator=mitigator)
        )
        logger.info("✅ Clipboard Monitor enabled.")

    if modules_cfg.get("visual", True):
        monitors.append(ActiveWindowMonitor(config, notifier, ai_client=ai_client))
        logger.info("✅ Visual Sentry (Active Window) enabled.")

    if modules_cfg.get("network", True):
        monitors.append(NetworkMonitor(config, notifier, mitigator=mitigator))
        logger.info("✅ Network Monitor enabled.")

    # 其他預設開啟或必要的監控
    monitors.append(KeystrokeMonitor(config, notifier))
    monitors.append(SystemResourceMonitor(config, notifier, mitigator=mitigator))

    for m in monitors:
        m.start()

    heartbeat = SystemHeartbeat(monitors)
    heartbeat.start()

    logger.info(i18n.get("system_starting") + " (Modules initialized)")

    try:
        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        logger.info(i18n.get("shutdown_gracefully"))
        for m in monitors:
            m.stop()
        heartbeat.stop()
    except Exception as e:
        logger.error(i18n.get("unexpected_error", error=e), exc_info=True)


if __name__ == "__main__":
    main()
