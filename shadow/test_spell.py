from base import BaseAPITestCase

class SpellTests(BaseAPITestCase):
    
    def test_spell_cast_and_check(self):
        response = self._post("/spell/cast", {
            "username": self.username
        })
        key = response.get('key')
        spell1 = response.get('spell')
        
        response2 = self._post("/spell/check", {
            "username": self.username,
            "token": self.token,
            "key": key
        })
        self.assertEqual(spell1, response2.get('raw'))
