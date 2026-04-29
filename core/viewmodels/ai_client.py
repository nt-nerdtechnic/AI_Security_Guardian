import logging

logger = logging.getLogger("Aegis_Guardian")

try:
    from guardian_brain import (
        analyze_visual_threat,
        analyze_command_semantics,
        analyze_command_semantics_detailed,
        refresh_ai_settings,
        VISUAL_MODEL,
        SEMANTIC_MODEL,
    )

    AI_AVAILABLE = True
except ImportError as _e:
    AI_AVAILABLE = False
    logger.warning(
        f"⚠️  [AI Brain] 無法載入 guardian_brain 模組: {_e}，AI 分析功能停用。"
    )


class AiBrainViewModel:
    """
    (ViewModel / Service)
    封裝 AI 分析函數的呼叫，並提供優雅降級機制。
    """

    def __init__(self):
        self._available = AI_AVAILABLE
        self.semantic_model = SEMANTIC_MODEL if self._available else ""
        self.visual_model = VISUAL_MODEL if self._available else ""
        if self._available:
            settings = refresh_ai_settings()
            self.semantic_model = settings.get("semantic_model", SEMANTIC_MODEL)
            self.visual_model = settings.get("visual_model", VISUAL_MODEL)
            logger.info("🧠 [AI Brain] 模組載入成功，AI 分析功能已啟用。")
            logger.info(
                "   視覺模型：%s  語義模型：%s",
                self.visual_model,
                self.semantic_model,
            )
        else:
            logger.warning(
                "⚠️  [AI Brain] guardian_brain 模組未載入，AI 分析功能停用。"
            )

    @property
    def available(self) -> bool:
        return self._available

    def analyze_semantic(self, text: str) -> bool:
        """語義分析：回傳 True 表示偵測到威脅"""
        if not self._available:
            return False
        try:
            return analyze_command_semantics(text)
        except Exception as e:
            logger.debug(f"[AI Brain] 語義分析失敗: {e}")
        return False

    def analyze_semantic_detailed(self, text: str) -> dict:
        """語義分析：回傳 verdict/confidence/category/reason/recommended_action"""
        if not self._available:
            return {"verdict": "safe", "confidence": 0.0}
        try:
            return analyze_command_semantics_detailed(text)
        except Exception as e:
            logger.debug(f"[AI Brain] 語義分析失敗: {e}")
        return {"verdict": "safe", "confidence": 0.0}

    def analyze_visual(self, image_path: str) -> bool:
        """視覺分析：回傳 True 表示偵測到威脅"""
        if not self._available:
            return False
        try:
            return analyze_visual_threat(image_path)
        except Exception as e:
            logger.debug(f"[AI Brain] 視覺分析失敗: {e}")
        return False
