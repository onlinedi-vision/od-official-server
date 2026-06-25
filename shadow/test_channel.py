from base import BaseAPITestCase

class ChannelTests(BaseAPITestCase):

    def test_create_channel(self):
        self.create_test_server()
        
        response = self.create_test_channel()
        self.assertIsNotNone(response.get('token'))

    def test_create_channel_max_length(self):
        self.create_test_server()

        long_channel = "flajkaldjflhkcvjhxzoyuafhldasjhfiocuzxgvhadfhsojk"
        response = self._post(f"/servers/{self.server_id}/create_channel", {
            "username": self.username,
            "token": self.token,
            "channel_name": long_channel
        })
        self.assertRegex(
            response.get('raw', ''),
            r'^Failed to create channel: Channel name longer than'
        )

    def test_create_channel_inexistent_server(self):
        response = self._post("/servers/a/create_channel", {
            "username": self.username,
            "token": self.token,
            "channel_name": "main"
        })
        self.assertEqual(response.get('raw'), "Couldn't find that server. (a) :(")
    
    def test_get_channels(self):
        self.create_test_server()
        self.create_test_channel()
        
        response = self._post(f"/servers/{self.server_id}/get_channels", {
            "username": self.username,
            "token": self.token
        })

        channel_name = response.get(
            'c_list', 
            [{}]
        )[1].get('channel_name') \
        if len(response.get('c_list', [])) > 1 \
        else None

        self.assertEqual(channel_name, "main")
