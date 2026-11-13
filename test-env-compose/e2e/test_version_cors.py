import os
import unittest

import requests

TIMEOUT = 10
HOST = os.environ.get("API_HOST", "http://localhost:1313")


class VersionCorsTests(unittest.TestCase):
    def test_version_cors_allows_landing0(self):
        origin = "https://landing0.onlinedi.vision"
        response = requests.get(
            f"{HOST}/version",
            headers={"Origin": origin},
            timeout=TIMEOUT,
        )
        self.assertTrue(response.ok)
        self.assertEqual(response.headers.get("Access-Control-Allow-Origin"), origin)
        self.assertEqual(response.text, "v0.1.0")

    def test_version_cors_blocks_example_com(self):
        origin = "https://example.com"
        response = requests.get(
            f"{HOST}/version",
            headers={"Origin": origin},
            timeout=TIMEOUT,
        )
        self.assertTrue(response.ok)
        self.assertIsNone(response.headers.get("Access-Control-Allow-Origin"))

