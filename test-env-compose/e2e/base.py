import unittest
import random
import string
import os
import requests

TIMEOUT=10

class BaseAPITestCase(unittest.TestCase):
    HOST = os.environ.get('API_HOST', 'http://localhost:1313')
    PASSWORD = os.environ.get('QA_E2E_ACCOUNT_PASSWORD', 'testpassword123')

    def setUp(self):
        self.random_suffix = ''.join(random.choices(string.ascii_lowercase + string.digits, k=12))
        self.username = self.random_suffix
        self.token = None
        self.server_id = None
        self.channel_name = "main"

        response = self._post("/new_user", {
            "username": self.username,
            "password": self.PASSWORD,
            "email": "test@example.com"
        })
        self.token = response.get('token')
        self.assertIsNotNone(self.token, f"Failed to create user {self.username}")

    def _request(self, method: str, endpoint: str, data = None) -> dict:
        url = f"{self.HOST}{endpoint}"

        version = requests.get(f"{self.HOST}/version", timeout=TIMEOUT)
        self.assertTrue(version.ok)

        try:
            if method == "GET":
                response = requests.get(url, timeout=TIMEOUT)
            elif method == "POST":
                response = requests.post(url, json=data, timeout=TIMEOUT)
            elif method == "PATCH":
                response = requests.patch(url, json=data, timeout=TIMEOUT)
            else:
                raise ValueError(f"Unsupported method: {method}")

            # response.raise_for_status()
            return response.json()
        except requests.exceptions.JSONDecodeError:
            return {"raw": response.text}
        except requests.exceptions.RequestException as e:
            return {"raw": str(e)}

    def _get(self, endpoint: str) -> dict:
        return self._request("GET", endpoint)

    def _post(self, endpoint: str, data: dict) -> dict:
        return self._request("POST", endpoint, data)

    def _patch(self, endpoint: str, data: dict) -> dict:
        return self._request("PATCH", endpoint, data)

    def create_test_server(self, name="QA_TEST_SERVER"):
        response = self._post("/create_server", {
            "username": self.username,
            "token": self.token,
            "desc": "L",
            "name": name,
            "img_url": "L"
        })
        self.token = response.get('token')
        self.server_id = response.get('sid')
        return response

    def create_test_channel(self, channel_name="main"):
        return self._post(f"/servers/{self.server_id}/create_channel", {
            "username": self.username,
            "token": self.token,
            "channel_name": channel_name
        })

    def create_second_user(self):
        username2 = f"qa_user2_{self._testMethodName}_{self.random_suffix}"
        response = self._post("/new_user", {
            "username": username2,
            "password": self.PASSWORD,
            "email": "test2@example.com"
        })
        return username2, response.get('token')
