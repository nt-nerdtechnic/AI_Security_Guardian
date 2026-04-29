import unittest

from guardian_brain import parse_semantic_analysis


class GuardianBrainSchemaTests(unittest.TestCase):
    def test_parse_semantic_json_schema(self):
        result = parse_semantic_analysis(
            '{"verdict":"threat","confidence":0.91,"category":"credential",'
            '"reason":"contains an API key","recommended_action":"redact"}'
        )

        self.assertEqual(result["verdict"], "threat")
        self.assertEqual(result["confidence"], 0.91)
        self.assertEqual(result["category"], "credential")
        self.assertEqual(result["recommended_action"], "redact")

    def test_parse_legacy_yes_no_response(self):
        result = parse_semantic_analysis("YES - reverse shell command")

        self.assertEqual(result["verdict"], "threat")
        self.assertEqual(result["category"], "legacy_text")


if __name__ == "__main__":
    unittest.main()
