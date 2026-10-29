"""HTTP integration/security checks. Use only the disposable test stack; never resets a database."""
import os
import unittest
import uuid
import httpx

BASE=os.getenv('TEST_API_URL','http://localhost:18080')
def uid(): return str(uuid.uuid4())

class PlatformFlow(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.client=httpx.Client(base_url=BASE,timeout=30,headers={"Origin":os.getenv("TEST_FRONTEND_ORIGIN","http://localhost:13000")})
        cls.prefix=uid()[:8]
        def signup(name):
            response=cls.client.post('/auth/signup',json={'organization_name':f'Test {name} {cls.prefix}','email':f'{name}-{cls.prefix}@example.test','full_name':name,'password':'Test-password-2026!'})
            response.raise_for_status();return response.json()
        cls.a=signup('Alpha');cls.b=signup('Beta')
        cls.ah={'Authorization':'Bearer '+cls.a['token']};cls.bh={'Authorization':'Bearer '+cls.b['token']}
        invite=cls.client.post('/invitations',headers=cls.ah,json={'email':f'customer-{cls.prefix}@example.test','role':'customer'}).json()
        customer=cls.client.post('/auth/accept-invitation',json={'token':invite['token'],'email':f'customer-{cls.prefix}@example.test','full_name':'Customer','password':'Test-customer-2026!'})
        customer.raise_for_status();cls.customer=customer.json();cls.ch={'Authorization':'Bearer '+cls.customer['token']}
        cls.t=cls.client.post('/tickets',headers=cls.ch,json={'subject':'Refund policy','description':'Refund policy'}).json()
        cls.other=cls.client.post('/tickets',headers=cls.bh,json={'subject':'Secret Beta','description':'Cross-tenant secret'}).json()
        cls.client.cookies.clear()

    def call(self,method,path,headers=None,**kw):return self.client.request(method,path,headers=headers or self.ah,**kw)

    def test_01_auth_and_membership(self):
        self.assertEqual(self.client.get('/me').status_code,401)
        response=self.client.post('/auth/signup',json={'organization_id':self.a['org_id'],'email':f'evil-{self.prefix}@example.test','password':'Valid-password-2026','full_name':'Attacker','role':'agent'})
        self.assertEqual(response.status_code,400)
        self.assertEqual(self.call('PATCH',f"/users/{self.a['user_id']}/role",json={'role':'agent'}).status_code,400)

    def test_02_ticket_boundaries(self):
        for suffix in ['', '/messages']:
            self.assertEqual(self.call('GET',f"/tickets/{self.t['id']}{suffix}",self.bh).status_code,404)
        self.assertEqual(self.call('PATCH',f"/tickets/{self.t['id']}",self.ch,json={'status':'solved'}).status_code,403)
        self.assertEqual(self.call('PATCH',f"/tickets/{self.t['id']}",json={'assigned_to':self.b['user_id']}).status_code,404)
        self.assertEqual(self.call('PATCH',f"/tickets/{self.t['id']}",json={'assigned_to':self.a['user_id']}).status_code,200)
        unassigned=self.call('PATCH',f"/tickets/{self.t['id']}",json={'unassign':True})
        self.assertEqual(unassigned.status_code,200);self.assertIsNone(unassigned.json()['assigned_to'])
        self.assertFalse(any(t['id']==self.other['id'] for t in self.call('GET','/inbox').json()))

    def test_03_message_visibility_and_idempotency(self):
        path=f"/tickets/{self.t['id']}/messages"
        payload={'body':'Private internal context','internal':True,'idempotency_key':uid()}
        first=self.call('POST',path,json=payload);self.assertEqual(first.status_code,200,first.text)
        duplicate=self.call('POST',path,json=payload);self.assertEqual(first.json()['id'],duplicate.json()['id'])
        changed=self.call('POST',path,json=payload|{'body':'Different payload'});self.assertEqual(changed.status_code,400)
        self.assertEqual(self.call('POST',path,self.ch,json=payload|{'idempotency_key':uid()}).status_code,403)
        self.assertFalse(any(m['internal'] for m in self.call('GET',path,self.ch).json()))
        response=self.call('POST',path,json={'body':'Your request is being reviewed.','internal':False,'idempotency_key':uid()})
        self.assertEqual(response.json()['delivery'],'portal_visible')
        self.assertTrue(any('reviewed' in m['body'] for m in self.call('GET',path,self.ch).json()))
        self.assertEqual(self.call('POST',f"/tickets/{self.other['id']}/messages",json={'body':'bad','internal':False,'idempotency_key':uid()}).status_code,404)

    def test_04_knowledge_and_grounding(self):
        for suffix in ['/summary','/category']:
            result=self.call('GET',f"/tickets/{self.t['id']}{suffix}")
            self.assertEqual(result.status_code,200,result.text)
            self.assertEqual(self.call('GET',f"/tickets/{self.t['id']}{suffix}",self.bh).status_code,404)
            self.assertEqual(self.call('GET',f"/tickets/{self.t['id']}{suffix}",self.ch).status_code,403)
        article=self.call('POST','/knowledge',json={'title':'Refund policy','content':'Refund policy requests are reviewed within five business days.','visibility':'team'});self.assertEqual(article.status_code,200,article.text)
        article_id=article.json()['id']
        response=self.call('POST',f"/tickets/{self.t['id']}/grounded-draft");self.assertEqual(response.status_code,200,response.text)
        self.assertFalse(response.json()['abstained']);self.assertTrue(response.json()['citations'])
        proposal=response.json()
        sent=self.call('POST',f"/tickets/{self.t['id']}/messages",json={'body':proposal['draft'],'proposal_id':proposal['id'],'internal':False,'idempotency_key':uid()})
        self.assertEqual(sent.status_code,200,sent.text)
        self.assertEqual(self.call('GET','/knowledge',self.bh).json(),[])
        self.assertEqual(self.call('GET','/knowledge',self.ch).json(),[])
        self.assertEqual(self.call('DELETE',f'/knowledge/{article_id}',self.bh).status_code,404)
        self.assertEqual(self.call('POST',f'/knowledge/{article_id}/reindex').status_code,200)
        self.assertEqual(self.call('DELETE',f'/knowledge/{article_id}').status_code,200)
        stale=self.call('POST',f"/tickets/{self.t['id']}/messages",json={'body':proposal['draft'],'proposal_id':proposal['id'],'internal':False,'idempotency_key':uid()})
        self.assertEqual(stale.status_code,400)
        draft=self.call('POST',f"/tickets/{self.t['id']}/grounded-draft").json();self.assertTrue(draft['abstained'])
        self.assertEqual(self.call('POST',f"/tickets/{self.t['id']}/grounded-draft",self.bh).status_code,404)
        self.assertEqual(self.call('POST',f"/tickets/{self.t['id']}/grounded-draft",self.ch).status_code,403)
        public=self.call('POST','/knowledge',json={'title':'Getting started','content':'Welcome to the help center.','visibility':'public'});self.assertEqual(public.status_code,200)
        me=self.call('GET','/me').json();help_result=self.client.get('/public/help/'+me['org_slug']);self.assertEqual(len(help_result.json()),1)

    def test_05_tools_and_approval(self):
        payload={'ticket_id':self.t['id'],'action':'ticket.label','arguments':{'label':'verified-test'},'idempotency_key':uid()}
        before=self.call('GET',f"/tickets/{self.t['id']}").json()
        preview=self.call('POST','/tools/preview',json=payload);self.assertTrue(preview.json()['dry_run'])
        self.assertEqual(self.call('GET',f"/tickets/{self.t['id']}").json()['labels'],before['labels'])
        first=self.call('POST','/tools',json=payload);self.assertEqual(first.status_code,200,first.text)
        self.assertEqual(first.json()['state'],'executed');self.assertEqual(first.json()['id'],self.call('POST','/tools',json=payload).json()['id'])
        self.assertEqual(self.call('POST','/tools',self.bh,json=payload).status_code,404)
        self.assertEqual(self.call('POST','/tools',self.ch,json=payload).status_code,403)
        self.assertEqual(self.call('POST','/tools',json=payload|{'arguments':{'label':'x','execute':'refund'}}).status_code,400)
        sensitive=self.call('POST','/tools',json={'ticket_id':self.t['id'],'action':'refund','arguments':{'reason':'Customer requested review'},'idempotency_key':uid()}).json()
        self.assertEqual(sensitive['state'],'pending')
        self.assertEqual(self.call('POST',f"/approvals/{sensitive['id']}",self.bh,json={'approve':True}).status_code,404)
        decided=self.call('POST',f"/approvals/{sensitive['id']}",json={'approve':True}).json()
        self.assertEqual(decided['state'],'blocked');self.assertFalse(decided['executed'])
        self.assertEqual(self.call('POST',f"/approvals/{sensitive['id']}",json={'approve':True}).status_code,400)

    def test_06_workflows(self):
        definition={'nodes':[{'id':'start','kind':'trigger','next':'tag'},{'id':'tag','kind':'action','value':'label:workflow-test','next':None}]}
        w=self.call('POST','/workflows',json={'name':'Test workflow','definition':definition});self.assertEqual(w.status_code,200,w.text);wid=w.json()['id']
        request={'ticket_id':self.t['id'],'simulation':True,'idempotency_key':uid()}
        simulation=self.call('POST',f'/workflows/{wid}/run',json=request);self.assertEqual(simulation.status_code,200,simulation.text)
        self.assertNotIn('workflow-test',self.call('GET',f"/tickets/{self.t['id']}").json()['labels'])
        self.assertEqual(self.call('POST',f'/workflows/{wid}/publish',self.bh).status_code,404)
        self.assertEqual(self.call('POST',f'/workflows/{wid}/publish').status_code,200)
        self.assertEqual(len(self.call('GET',f'/workflows/{wid}/versions').json()),1)
        self.assertEqual(self.call('GET',f'/workflows/{wid}/versions',self.bh).status_code,404)
        self.assertEqual(self.call('POST',f'/workflows/{wid}/rollback',self.bh,json={'version':1}).status_code,404)
        restored=self.call('POST',f'/workflows/{wid}/rollback',json={'version':1})
        self.assertEqual(restored.status_code,200,restored.text)
        self.assertFalse(restored.json()['published_changed'])
        request|={'simulation':False,'idempotency_key':uid()}
        run=self.call('POST',f'/workflows/{wid}/run',json=request);self.assertEqual(run.status_code,200,run.text)
        self.assertEqual(run.json()['state'],'completed')
        self.assertEqual(run.json()['id'],self.call('POST',f'/workflows/{wid}/run',json=request).json()['id'])
        self.assertIn('workflow-test',self.call('GET',f"/tickets/{self.t['id']}").json()['labels'])
        self.assertEqual(self.call('POST',f'/workflows/{wid}/run',self.bh,json=request).status_code,404)
        definition['nodes'][1]['next']='start'
        self.assertEqual(self.call('POST','/workflows',json={'name':'cycle','definition':definition}).status_code,400)
        self.assertEqual(self.call('GET','/workflow-runs',self.bh).json(),[])

    def test_07_analytics_and_admin(self):
        metrics=self.call('GET','/analytics');self.assertEqual(metrics.status_code,200,metrics.text)
        self.assertEqual(metrics.json()['totals']['total'],1)
        self.assertGreaterEqual(metrics.json()['ai']['proposals'],2)
        self.assertEqual(self.call('GET','/analytics',self.ch).status_code,403)
        self.assertEqual(self.call('GET','/audit',self.ch).status_code,403)
        self.assertTrue(self.call('GET','/audit').json())
        self.assertFalse(self.call('GET','/billing').json()['charges_enabled'])
        self.assertTrue(all(i['status']!='active' for i in self.call('GET','/integrations').json() if i['id'] in ['email','stripe','whatsapp']))

    def test_08_status_machine_and_csat(self):
        path=f"/tickets/{self.t['id']}"
        self.assertEqual(self.call('PATCH',path,json={'status':'closed'}).status_code,400)
        self.assertEqual(self.call('PATCH',path,json={'status':'solved'}).status_code,200)
        self.assertEqual(self.call('POST',path+'/csat',self.ch,json={'score':5}).status_code,200)
        self.assertEqual(self.call('PATCH',path,json={'status':'closed'}).status_code,200)
        self.assertEqual(self.call('PATCH',path,json={'status':'open'}).status_code,400)
        self.assertEqual(self.call('POST',path+'/messages',json={'body':'after close','internal':False,'idempotency_key':uid()}).status_code,400)

    def test_09_sandbox_roundtrip_and_notifications(self):
        email=f'customer-{self.prefix}@example.test'
        payload={'customer_email':email,'subject':'Sandbox question','body':'Please help with sandbox email.','idempotency_key':uid()}
        inbound=self.call('POST','/sandbox/email',json=payload);self.assertEqual(inbound.status_code,200,inbound.text)
        tid=inbound.json()['ticket_id']
        self.assertEqual(self.call('POST','/sandbox/email',json=payload).json()['id'],inbound.json()['id'])
        response=self.call('POST',f'/tickets/{tid}/messages',json={'body':'Sandbox answer','internal':False,'idempotency_key':uid()})
        self.assertEqual(response.status_code,200,response.text)
        mailbox=self.call('GET','/sandbox/mailbox').json()
        self.assertTrue(any(m['body']=='Sandbox answer' and m['direction']=='outbound' and m['state']=='sandbox_only' for m in mailbox))
        self.assertFalse(any(m['ticket_id']==tid for m in self.call('GET','/sandbox/mailbox',self.bh).json()))
        self.assertEqual(self.call('GET','/sandbox/mailbox',self.ch).status_code,403)
        import time
        notes=[]
        for _ in range(20):
            notes=self.call('GET','/notifications').json()
            if notes:break
            time.sleep(.2)
        self.assertTrue(notes)
        self.assertFalse(any(n['id']==notes[0]['id'] for n in self.call('GET','/notifications',self.bh).json()))
        self.assertEqual(self.call('POST',f"/notifications/{notes[0]['id']}/read",self.bh).status_code,404)
        self.assertEqual(self.call('POST',f"/notifications/{notes[0]['id']}/read").status_code,200)
        self.assertEqual(self.call('GET','/notifications',self.ch).status_code,403)

    def test_11_concurrent_message_and_workflow_idempotency(self):
        from concurrent.futures import ThreadPoolExecutor
        ticket=self.call('POST','/tickets',self.ch,json={'subject':'Concurrent test','description':'Synthetic concurrency fixture'}).json()
        definition={'nodes':[{'id':'start','kind':'trigger','next':'tag'},{'id':'tag','kind':'action','value':'label:once'}]}
        workflow=self.call('POST','/workflows',json={'name':'Concurrent run','definition':definition}).json()
        self.assertEqual(self.call('POST',f"/workflows/{workflow['id']}/publish").status_code,200)
        message={'body':'One logical message','internal':False,'idempotency_key':uid()}
        run={'ticket_id':ticket['id'],'simulation':False,'idempotency_key':uid()}
        def post(index):
            return self.call('POST',f"/tickets/{ticket['id']}/messages" if index%2 else f"/workflows/{workflow['id']}/run",json=message if index%2 else run)
        with ThreadPoolExecutor(max_workers=8) as pool:results=list(pool.map(post,range(12)))
        self.assertTrue(all(r.status_code==200 for r in results),[(r.status_code,r.text) for r in results])
        self.assertEqual(len({r.json()['id'] for i,r in enumerate(results) if i%2}),1)
        self.assertEqual(len({r.json()['id'] for i,r in enumerate(results) if not i%2}),1)
        messages=self.call('GET',f"/tickets/{ticket['id']}/messages").json()
        self.assertEqual(sum(m['body']=='One logical message' for m in messages),1)

    def test_12_cookie_csrf_and_password_reset(self):
        email=f'Alpha-{self.prefix}@example.test'
        origin=os.getenv('TEST_FRONTEND_ORIGIN','http://localhost:13000')
        browser=httpx.Client(base_url=BASE,headers={'Origin':origin})
        login=browser.post('/auth/login',json={'email':email,'password':'Test-password-2026!'})
        self.assertEqual(login.status_code,200,login.text)
        self.assertIn('HttpOnly',login.headers['set-cookie']);self.assertIn('SameSite=Strict',login.headers['set-cookie'])
        self.assertEqual(browser.get('/me').status_code,200)
        denied=browser.post('/tickets',headers={'Origin':'http://attacker.example'},json={'subject':'CSRF'})
        self.assertEqual(denied.status_code,403)
        self.assertEqual(browser.post('/auth/logout').status_code,200)
        self.assertEqual(browser.get('/me').status_code,401)
        from urllib.parse import urlparse,parse_qs
        verification=next(m for m in self.call('GET','/sandbox/mailbox').json() if m['subject']=='Sandbox verify link')
        verify_token=parse_qs(urlparse(verification['body'].splitlines()[0]).query)['token'][0]
        self.assertEqual(self.call('POST','/auth/verify-email',json={'token':verify_token}).status_code,200)
        self.assertEqual(self.call('POST','/auth/verify-email',json={'token':verify_token}).status_code,400)
        response=self.call('POST','/auth/request-reset',json={'email':email});self.assertEqual(response.status_code,200,response.text)
        mailbox=self.call('GET','/sandbox/mailbox').json()
        reset=next(m for m in mailbox if m['subject']=='Sandbox reset link')
        token=parse_qs(urlparse(reset['body'].splitlines()[0]).query)['token'][0]
        result=self.call('POST','/auth/reset-password',json={'token':token,'password':'New-test-password-2026!'})
        self.assertEqual(result.status_code,200,result.text)
        self.assertEqual(self.call('GET','/me').status_code,401)
        self.assertEqual(self.call('POST','/auth/reset-password',json={'token':token,'password':'Other-password-2026!'}).status_code,400)
        login=self.client.post('/auth/login',json={'email':email,'password':'New-test-password-2026!'})
        self.assertEqual(login.status_code,200,login.text)
        self.assertEqual(self.client.get('/me',headers={'Authorization':'Bearer '+login.json()['token']}).status_code,200)
        self.client.cookies.clear()

if __name__=='__main__':unittest.main(verbosity=2)
