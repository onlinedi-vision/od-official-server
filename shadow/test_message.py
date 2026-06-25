import random
import string
import time
import datetime

from base import BaseAPITestCase

class MessageTests(BaseAPITestCase):
    
    def test_send_message(self):
        self.create_test_server()
        self.create_test_channel()
        
        message = "This is the sent message."
        response = self._post(f"/servers/{self.server_id}/main/send_message", {
            "username": self.username,
            "token": self.token,
            "m_content": message
        })
        self.assertEqual(response.get('raw'), "Message sent.")
    
    def test_send_message_max_length(self):
        self.create_test_server()
        self.create_test_channel()
        
        long_message = ''.join(random.choices(string.ascii_letters + string.digits, k=3001))
        response = self._post(f"/servers/{self.server_id}/main/send_message", {
            "username": self.username,
            "token": self.token,
            "m_content": long_message
        })
        self.assertRegex(response.get('raw', ''), r'^Failed to send message: Message longer than ')
    
    def test_send_message_inexistent_server(self):
        response = self._post("/servers/a/main/send_message", {
            "username": self.username,
            "token": self.token,
            "m_content": "test message"
        })
        self.assertEqual(response.get('raw'), "Couldn't find that server. (a) :(")
    
    def test_send_message_inexistent_channel(self):
        self.create_test_server()
        
        response = self._post(f"/servers/{self.server_id}/a/send_message", {
            "username": self.username,
            "token": self.token,
            "m_content": "test message"
        })
        self.assertEqual(response.get('raw'), "Couldn't find that channel. (a) :(")
    
    def test_get_messages(self):
        self.create_test_server()
        self.create_test_channel()
        
        message = "This is the sent message."
        self._post(f"/servers/{self.server_id}/main/send_message", {
            "username": self.username,
            "token": self.token,
            "m_content": message
        })
        
        response = self._post(f"/servers/{self.server_id}/main/get_messages_migration", {
            "username": self.username,
            "token": self.token,
            "limit": "100",
            "offset": "0"
        })
        received = response.get(
            'm_list', 
            [{}]
        )[0].get('m_content') if response.get('m_list') else None
        self.assertEqual(received, message)
    
    def test_delete_message(self):
        self.create_test_server()
        self.create_test_channel()
        
        message = "This is the sent message."
        self._post(f"/servers/{self.server_id}/main/send_message", {
            "username": self.username,
            "token": self.token,
            "m_content": message
        })
        
        get_response = self._post(f"/servers/{self.server_id}/main/get_messages_migration", {
            "username": self.username,
            "token": self.token,
            "limit": "100",
            "offset": "0"
        })
        datetime_str = get_response.get('m_list', [{}])[0].get('datetime', '')
        datetime_len = len(datetime_str) - 3
        api_datetime_lh = datetime_str[:datetime_len]
        api_datetime_rh = datetime_str[-3:]
        api_datetime = datetime.datetime.fromtimestamp(
            float(f"{api_datetime_lh}.{api_datetime_rh}")
        ).strftime('%Y-%m-%d %H:%M:%S')
        
        delete_response = self._post(f"/servers/{self.server_id}/main/delete_message", {
            "username": self.username,
            "token": self.token,
            "datetime": api_datetime
        })
        self.assertEqual(delete_response.get('raw'), "Message deleted successfully")
    
    def test_ttl_expiration(self):
        self.create_test_server()
        self.create_test_channel()
        
        self._patch("/user/ttl", {
            "username": self.username,
            "token": self.token,
            "ttl": "s"
        })
        
        short_ttl_message = "This message will be deleted after 3 seconds."
        self._post(f"/servers/{self.server_id}/main/send_message", {
            "username": self.username,
            "token": self.token,
            "m_content": short_ttl_message
        })
        
        time.sleep(1)
        
        response = self._post(f"/servers/{self.server_id}/main/get_messages_migration", {
            "username": self.username,
            "token": self.token,
            "limit": "100",
            "offset": "0"
        })
        
        messages = [m.get('m_content', '') for m in response.get('m_list', [])]
        self.assertNotIn(short_ttl_message, messages)
