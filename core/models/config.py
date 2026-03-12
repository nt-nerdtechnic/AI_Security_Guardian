import yaml
import logging
from pathlib import Path

# 使用與主目錄相同的基底路徑
BASE_DIR = Path(__file__).resolve().parent.parent.parent
CONFIG_FILE = BASE_DIR / "config.yaml"

logger = logging.getLogger("Aegis_Guardian")


class ConfigModel:
    """
    (Model) 負責讀取與解析 config.yaml 檔案
    """

    @staticmethod
    def load():
        if not CONFIG_FILE.exists():
            logger.error(f"Config file not found at {CONFIG_FILE}")
            return {}

        try:
            with open(CONFIG_FILE, "r", encoding="utf-8") as f:
                config = yaml.safe_load(f)
                return config or {}
        except Exception as e:
            logger.error(f"Failed to load configuration: {e}")
            return {}
