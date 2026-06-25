import random
import string
import time

from base import BaseAPITestCase

class AuthTests(BaseAPITestCase):
    
    def test_get_user_servers_null_token(self):
        response = self._post("/get_user_servers", {
            "username": self.username,
            "token": ""
        })
        self.assertEqual(response.get('raw'), "Invalid or expired token")
    
    def test_new_user_max_username_length(self):
        long_username = ''.join(random.choices(string.ascii_letters + string.digits, k=31))
        response = self._post("/new_user", {
            "username": long_username,
            "password": self.PASSWORD,
            "email": "test@example.com"
        })
        self.assertRegex(response.get('raw', ''), r'^Failed to create user: Username longer than ')
    
    def test_ttl_update(self):
        response = self._patch("/user/ttl", {
            "username": self.username,
            "token": self.token,
            "ttl": "s"
        })
        self.assertEqual(response.get('raw'), "TTL Updated.")
    
    def test_ttl_back_to_normal(self):
        self._patch("/user/ttl", {
            "username": self.username,
            "token": self.token,
            "ttl": "s"
        })
        
        response = self._patch("/user/ttl", {
            "username": self.username,
            "token": self.token,
            "ttl": "N"
        })
        self.assertEqual(response.get('raw'), "TTL Updated.")
    
    def test_token_ttl_expiration(self):
        self.create_test_server()
        
        self._patch("/user/ttl", {
            "username": self.username,
            "token": self.token,
            "ttl": "s"
        })
        
        time.sleep(2)
        
        response = self._post("/create_server", {
            "username": self.username,
            "token": self.token,
            "desc": "L",
            "name": "QA_TEST_SERVER_2",
            "img_url": "L"
        })
        self.assertEqual(response.get('raw'), "Invalid token")
