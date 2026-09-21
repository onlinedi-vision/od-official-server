from base import BaseAPITestCase

class UserTest(BaseAPITestCase):
    def test_empty_user_servers_list(self):
        response = self._post("/get_user_servers", {
            "username": self.username,
            "token": self.token
        })
        s_list = response.get("s_list")
        self.assertEqual(s_list, [])
