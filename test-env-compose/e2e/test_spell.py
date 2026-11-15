from concurrent.futures import ThreadPoolExecutor
from threading import Barrier

import requests

from base import BaseAPITestCase, TIMEOUT

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

        response3 = self._post("/spell/check", {
            "username": self.username,
            "token": self.token,
            "key": key
        })
        self.assertEqual("Could not find Spell...", response3.get('raw'))

    def test_concurrent_spell_checks_have_one_successful_claimant(self):
        cast_response = self._post("/spell/cast", {
            "username": self.username
        })
        key = cast_response.get('key')
        spell = cast_response.get('spell')
        self.assertIsNotNone(key)
        self.assertIsNotNone(spell)

        worker_count = 8
        barrier = Barrier(worker_count)
        payload = {
            "username": self.username,
            "token": self.token,
            "key": key
        }

        def check_spell(_):
            barrier.wait(timeout=TIMEOUT)
            return requests.post(
                f"{self.HOST}/spell/check", json=payload, timeout=TIMEOUT
            )

        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            responses = list(executor.map(check_spell, range(worker_count)))

        successful_responses = [response for response in responses if response.status_code == 200]
        self.assertEqual(1, len(successful_responses))
        self.assertEqual(spell, successful_responses[0].text)
        for response in responses:
            if response.status_code != 200:
                self.assertEqual(500, response.status_code)
                self.assertEqual("Could not find Spell...", response.text)
