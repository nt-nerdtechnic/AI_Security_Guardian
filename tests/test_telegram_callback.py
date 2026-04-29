import base64
import hashlib
import hmac
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import core.main as main

BOT_TOKEN = "123456:test-token"
CHAT_ID = "998877"


def signed(raw: str) -> str:
    digest = hmac.new(
        BOT_TOKEN.encode("utf-8"), raw.encode("utf-8"), hashlib.sha256
    ).digest()[:6]
    return f"{raw}|{base64.urlsafe_b64encode(digest).decode('ascii')}"


def callback(raw: str, sender_id: str = CHAT_ID) -> dict:
    return {
        "bot_token": BOT_TOKEN,
        "authorized_chat_id": CHAT_ID,
        "from": {"id": sender_id},
        "data": signed(raw),
        "message": {"message_id": 1, "chat": {"id": CHAT_ID}, "text": "alert"},
    }


class TelegramCallbackSecurityTests(unittest.TestCase):
    def test_valid_signature_is_accepted(self):
        self.assertTrue(main._verify_telegram_callback(callback("terminate|1234")))

    def test_wrong_sender_is_rejected(self):
        self.assertFalse(
            main._verify_telegram_callback(callback("terminate|1234", sender_id="1"))
        )

    def test_missing_signature_is_rejected(self):
        payload = callback("terminate|1234")
        payload["data"] = "terminate|1234"
        self.assertFalse(main._verify_telegram_callback(payload))

    def test_tampered_payload_is_rejected(self):
        payload = callback("terminate|1234")
        payload["data"] = payload["data"].replace("1234", "1")
        self.assertFalse(main._verify_telegram_callback(payload))

    def test_quarantine_requires_recorded_file_metadata(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            incident_path = tmp_path / "incidents.json"
            candidate = tmp_path / "payload.sh"
            candidate.write_text("echo suspicious", encoding="utf-8")
            incident_path.write_text(
                json.dumps(
                    {
                        "module": "FileIntegrity",
                        "severity": "WARNING",
                        "message": "checksum mismatch",
                        "metadata": {"file_path": str(candidate)},
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            with mock.patch.object(main, "INCIDENTS_JSON", incident_path):
                self.assertEqual(
                    main._find_recorded_file_path(str(candidate)), candidate
                )
                self.assertIsNone(
                    main._find_recorded_file_path(str(tmp_path / "other.sh"))
                )

    def test_terminate_blocks_critical_pid(self):
        result = main._terminate_pid_with_audit(1)
        self.assertIn("[Denied]", result)


if __name__ == "__main__":
    unittest.main()
