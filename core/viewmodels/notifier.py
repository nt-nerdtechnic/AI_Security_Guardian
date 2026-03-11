import logging
import requests
import time
import threading

logger = logging.getLogger('Aegis_Guardian')

class TelegramNotifierViewModel:
    """
    (ViewModel / Service)
    封裝所有向 Telegram 發送文字、截圖、互動鍵盤的邏輯。
    """
    def __init__(self, config):
        self.config = config.get('webhook', {}).get('telegram', {})
        self.bot_token = self.config.get('bot_token', '')
        self.chat_id = self.config.get('chat_id', '')

    @property
    def is_configured(self):
        return bool(self.bot_token and self.chat_id)

    def send_alert(self, message):
        """傳送文字訊息到指定 Telegram"""
        if not self.is_configured:
            logger.debug("Telegram credentials not configured. Skipping webhook.")
            return

        url = f"https://api.telegram.org/bot{self.bot_token}/sendMessage"
        payload = {"chat_id": self.chat_id, "text": message}
        try:
            resp = requests.post(url, json=payload, timeout=5)
            if resp.status_code == 200:
                logger.debug("Telegram alert sent.")
            else:
                logger.error(f"Telegram alert failed: {resp.text}")
        except Exception as e:
            logger.error(f"Telegram alert failed: {e}")

    def send_snapshot(self, filepath, caption=""):
        """透過 Telegram sendPhoto API 傳送截圖檔案"""
        if not self.is_configured:
            logger.debug("Telegram credentials not configured. Skipping snapshot upload.")
            return

        url = f"https://api.telegram.org/bot{self.bot_token}/sendPhoto"
        try:
            with open(filepath, 'rb') as photo:
                payload = {"chat_id": self.chat_id, "caption": caption[:1024]}
                resp = requests.post(url, data=payload, files={"photo": photo}, timeout=15)
            if resp.status_code == 200:
                logger.debug(f"Snapshot sent to Telegram: {filepath}")
            else:
                logger.error(f"Failed to send snapshot to Telegram: {resp.text}")
        except Exception as e:
            logger.error(f"Telegram snapshot upload error: {e}")

    def send_interactive_alert(self, message, buttons):
        """傳送帶有 Inline Keyboard 按鈕的訊息"""
        if not self.is_configured:
            logger.debug("Telegram credentials not configured. Skipping interactive alert.")
            return None

        url = f"https://api.telegram.org/bot{self.bot_token}/sendMessage"
        reply_markup = {"inline_keyboard": buttons}
        payload = {
            "chat_id": self.chat_id,
            "text": message,
            "reply_markup": reply_markup
        }
        try:
            resp = requests.post(url, json=payload, timeout=5)
            if resp.status_code == 200:
                logger.info("Interactive Telegram alert sent.")
                return resp.json().get('result', {}).get('message_id')
            else:
                logger.error(f"Failed to send interactive alert: {resp.text}")
        except Exception as e:
            logger.error(f"Telegram interactive alert error: {e}")
        return None

    def start_polling(self, callback_handler):
        """啟動監聽執行緒，處理 Telegram 按鈕回調"""
        if not self.bot_token:
            return

        def poll():
            offset = 0
            logger.info("Telegram Bot Polling started for remote approval.")
            while True:
                try:
                    url = f"https://api.telegram.org/bot{self.bot_token}/getUpdates?offset={offset}&timeout=30"
                    resp = requests.get(url, timeout=35)
                    if resp.status_code == 200:
                        updates = resp.json().get('result', [])
                        for update in updates:
                            offset = update['update_id'] + 1
                            if 'callback_query' in update:
                                # Inject bot_token for editMessageText usage later
                                query = update['callback_query']
                                query['bot_token'] = self.bot_token
                                callback_handler(query)
                    else:
                        time.sleep(5)
                except Exception as e:
                    logger.debug(f"Polling error: {e}")
                    time.sleep(10)

        threading.Thread(target=poll, daemon=True).start()
