import json
import logging
from pathlib import Path

logger = logging.getLogger('Aegis_Guardian')

class I18nManager:
    """
    Internationalization Manager for Aegis Guardian.
    Loads language string packs from locales/ directory.
    """
    def __init__(self, language="zh-TW"):
        self.language = language
        self.strings = {}
        self.base_dir = Path(__file__).resolve().parent.parent / 'locales'
        self.load_language(language)

    def load_language(self, language):
        locale_file = self.base_dir / f"{language}.json"
        
        # Fallback to en if requested language doesn't exist
        if not locale_file.exists():
            logger.warning(f"Locale file not found: {locale_file}. Falling back to 'en'.")
            locale_file = self.base_dir / "en.json"
            if not locale_file.exists():
                logger.error(f"Fallback locale file also not found: {locale_file}")
                return
                
        try:
            with open(locale_file, 'r', encoding='utf-8') as f:
                self.strings = json.load(f)
            logger.info(f"Loaded language pack: {locale_file.name}")
        except Exception as e:
            logger.error(f"Failed to load language pack {locale_file.name}: {e}")

    def get(self, key, **kwargs):
        """
        Get a string by key, and format it with kwargs if provided.
        Returns the key itself if the string is not found.
        """
        template = self.strings.get(key)
        if template is None:
            return f"[{key}]"
        
        try:
            return template.format(**kwargs)
        except KeyError as e:
            logger.warning(f"Missing format key {e} for string '{key}'")
            return template
        except Exception as e:
            logger.warning(f"Format error for string '{key}': {e}")
            return template
