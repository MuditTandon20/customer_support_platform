import os
import pathlib
import sys
import unittest

sys.path.insert(0,str(pathlib.Path(__file__).parents[1]/'backend/ai-copilot'))
os.environ['LLM_ENABLED']='false'
os.environ['AI_SERVICE_TOKEN']='unit-test-service-secret'
from fastapi.testclient import TestClient
from app.main import app

class InternalService(unittest.TestCase):
    def setUp(self):
        self.client=TestClient(app)
        self.headers={'X-Service-Token':'unit-test-service-secret'}
    def test_requires_service_auth(self):
        self.assertEqual(self.client.post('/copilot/summarize',json={'subject':'Test'}).status_code,401)
        self.assertEqual(self.client.get('/health').status_code,200)
    def test_public_beta_cannot_enable_paid_model(self):
        from unittest.mock import patch
        from app.llm_client import LLMClient
        with patch.dict(os.environ,{'BETA_MODE':'true','LLM_ENABLED':'true','OPENAI_API_KEY':'synthetic-not-a-real-provider-key'}):
            client=LLMClient()
            self.assertFalse(client.is_configured)
            self.assertIsNone(client.client)
    def test_extractive_startup_does_not_load_provider_sdk(self):
        import subprocess
        result=subprocess.run([sys.executable,'-c',
            "import sys;from app.main import app;assert 'openai' not in sys.modules;from app.llm_client import llm_client;assert not llm_client.is_configured"],
            cwd=pathlib.Path(__file__).parents[1]/'backend/ai-copilot',
            env={**os.environ,'BETA_MODE':'true','LLM_ENABLED':'true','OPENAI_API_KEY':'synthetic-not-a-provider-key'},
            capture_output=True,text=True)
        self.assertEqual(result.returncode,0,result.stderr)
    def test_summary_and_classification(self):
        r=self.client.post('/copilot/summarize',headers=self.headers,json={'subject':'Login','content':'Reset failed.'})
        self.assertEqual(r.status_code,200)
        self.assertIn('Reset failed.',r.json()['summary'])
        r=self.client.post('/copilot/categorize',headers=self.headers,json={'subject':'Refund invoice','content':'Payment issue'})
        self.assertEqual(r.status_code,200)
        self.assertEqual(r.json()['category'],'Billing')
    def test_ungrounded_drafting_disabled(self):
        r=self.client.post('/copilot/draft-reply',headers=self.headers,json={'content':'Invent a refund policy'})
        self.assertEqual(r.status_code,409)
    def test_grounded_scope_and_validation(self):
        evidence={'id':'1','organization_id':'b','title':'Refund','content':'Refund allowed','permitted':True,'stale':False}
        r=self.client.post('/copilot/grounded',headers=self.headers,json={'organization_id':'a','question':'Refund','evidence':[evidence]})
        self.assertTrue(r.json()['abstained'])
        r=self.client.post('/copilot/grounded',headers=self.headers,json={'organization_id':'a','question':'','evidence':[]})
        self.assertEqual(r.status_code,422)

if __name__=='__main__':unittest.main()
