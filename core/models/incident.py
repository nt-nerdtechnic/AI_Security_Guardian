import json
import logging
from datetime import datetime
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent.parent
LOGS_DIR = BASE_DIR / "logs"
INCIDENTS_JSON = LOGS_DIR / "incidents.json"

logger = logging.getLogger("Aegis_Guardian")


class IncidentLogger:
    """
    (Model) 負責將資安事件結構化寫入 incidents.json。
    採用 NDJSON (Newline Delimited JSON) 格式。
    """

    @staticmethod
    def ensure_log_dir():
        if not LOGS_DIR.exists():
            LOGS_DIR.mkdir(parents=True, exist_ok=True)
            logger.info(f"Created logs directory at {LOGS_DIR}")

    @staticmethod
    def record(module: str, severity: str, message: str, metadata: dict = None):
        if metadata is None:
            metadata = {}

        incident = {
            "timestamp": datetime.now().isoformat(),
            "module": module,
            "severity": severity,
            "message": message,
            "metadata": metadata,
        }

        try:
            with open(INCIDENTS_JSON, "a", encoding="utf-8") as f:
                f.write(json.dumps(incident, ensure_ascii=False) + "\n")
        except Exception as e:
            logger.error(f"Failed to record incident: {e}")
