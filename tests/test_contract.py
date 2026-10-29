import json
import pathlib
import re
import unittest

ROOT=pathlib.Path(__file__).parents[1]
class ApiContract(unittest.TestCase):
    def test_inventory_matches_rust_routes(self):
        source=(ROOT/'backend/rust-api/src/main.rs').read_text(encoding='utf-8')
        actual={re.sub(r':(\w+)',r'{\1}',p) for p in re.findall(r'\.route\(\s*"([^"]+)"',source)}
        contract=json.loads((ROOT/'backend/rust-api/openapi.json').read_text(encoding='utf-8'))
        self.assertEqual(actual,set(contract['paths']))
        self.assertEqual(contract['openapi'],'3.1.0')
        self.assertIn('post',contract['paths']['/tickets/{id}/messages'])
        self.assertFalse(contract['components']['schemas']['Reply']['additionalProperties'])
    def test_sensitive_tool_schema_is_explicit(self):
        contract=json.loads((ROOT/'backend/rust-api/openapi.json').read_text(encoding='utf-8'))
        names=contract['components']['schemas']['Tool']['properties']['action']['enum']
        self.assertIn('refund',names)
        self.assertNotIn('shell.exec',names)
        self.assertIn('idempotency_key',contract['components']['schemas']['Tool']['required'])
if __name__=='__main__':unittest.main()
