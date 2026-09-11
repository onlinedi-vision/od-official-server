from base import BaseAPITestCase

class ServerTestsV1(BaseAPITestCase):
    def test_server_info(self):
        self.create_test_server(name="my server", desc="desc", img_url="img_url")

        response = self._get(f"/v1/servers/{self.server_id}/info")

        self.assertEqual(response.get("desc"), "desc")
        self.assertEqual(response.get("img_url"), "img_url")
        self.assertEqual(response.get("name"), "my server")

        # fe_config should still appear as empty even on old servers created with
        # v0 create_server...
        self.assertEqual(response.get("fe_config"), "")

    def test_v1_server_info(self):
        self.create_test_server(
            name="my server", desc="desc",
            img_url="img_url", create_server_endpoint="/v1/create_server")

        response = self._get(f"/v1/servers/{self.server_id}/info")

        self.assertEqual(response.get("desc"), "desc")
        self.assertEqual(response.get("img_url"), "img_url")
        self.assertEqual(response.get("name"), "my server")
        self.assertEqual(response.get("fe_config"), "")

    def test_patch_server_config(self):
        self.create_test_server()
        response = self._patch(f"/v1/servers/{self.server_id}/frontend", {
            "username": self.username,
            "token": self.token,
            "config": """{"color":"red"}"""
        })
        self.assertEqual(response.get("raw", ""), "Frontend configuration updated!")

        response = self._get(f"/v1/servers/{self.server_id}/frontend")
        self.assertEqual(response.get("fe_config"), """{"color":"red"}""")
