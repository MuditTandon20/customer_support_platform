"""Stage 1 real-data contract tests against a disposable native test stack."""
import os
import unittest
import uuid
import httpx

def key():return str(uuid.uuid4())
class Stage1(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.c=httpx.Client(base_url=os.getenv('TEST_API_URL','http://localhost:18080'),timeout=30,headers={'Origin':os.getenv('TEST_FRONTEND_ORIGIN','http://localhost:13000')})
        suffix=key()[:8]
        def signup(name):
            r=cls.c.post('/auth/signup',json={'organization_name':f'Stage1 synthetic {name} {suffix}','email':f'{name}-{suffix}@example.test','full_name':f'Synthetic {name}','password':'Stage1-test-password-2026!'})
            r.raise_for_status();return r.json()
        cls.a=signup('a');cls.b=signup('b');cls.ah={'Authorization':'Bearer '+cls.a['token']};cls.bh={'Authorization':'Bearer '+cls.b['token']}
        email=f'customer-{suffix}@example.test'
        invitation=cls.c.post('/invitations',headers=cls.ah,json={'email':email,'role':'customer'}).json()
        customer=cls.c.post('/auth/accept-invitation',json={'email':email,'token':invitation['token'],'full_name':'Synthetic customer','password':'Stage1-test-password-2026!'})
        customer.raise_for_status();cls.ch={'Authorization':'Bearer '+customer.json()['token']};cls.c.cookies.clear()
    def req(self,method,path,data=None,headers=None):
        r=self.c.request(method,path,json=data,headers=headers or self.ah)
        return r
    def ticket(self,subject='Synthetic issue',headers=None):
        r=self.req('POST','/tickets',{'subject':subject,'description':subject},headers=headers or self.ch);self.assertEqual(r.status_code,200,r.text);return r.json()
    def workflow(self,event,action):
        nodes=[{'id':'start','kind':'trigger','value':event,'next':'action'},{'id':'action','kind':'approval' if action=='approval' else 'action','value':'Human review' if action=='approval' else action}]
        r=self.req('POST','/workflows',{'name':f'Synthetic {event} {action}','definition':{'nodes':nodes}});self.assertEqual(r.status_code,200,r.text)
        wid=r.json()['id'];self.assertEqual(self.req('POST',f'/workflows/{wid}/publish',{}).status_code,200);return wid
    def test_01_chunk_ingestion_permissions_and_tail_retrieval(self):
        content=' '.join(f'term{i:04}' for i in range(650))+' uniquelysearchabletail'
        r=self.req('POST','/knowledge',{'title':'Synthetic long article','content':content,'visibility':'team'});self.assertEqual(r.status_code,200,r.text);doc=r.json()['id']
        chunks=self.req('GET',f'/knowledge/{doc}/chunks');self.assertEqual(chunks.status_code,200)
        parts=chunks.json();self.assertGreater(len(parts),3)
        self.assertTrue(all(len(c['content'].split())<=180 for c in parts))
        self.assertEqual(parts[0]['content'].split()[-30:],parts[1]['content'].split()[:30])
        self.assertEqual(self.req('GET',f'/knowledge/{doc}/chunks',headers=self.bh).status_code,404)
        self.assertEqual(self.req('GET',f'/knowledge/{doc}/chunks',headers=self.ch).status_code,404)
        t=self.ticket('uniquelysearchabletail')
        draft=self.req('POST',f"/tickets/{t['id']}/grounded-draft",{}).json()
        self.assertFalse(draft['abstained']);self.assertIn('uniquelysearchabletail',draft['draft']);self.assertTrue(draft['citations'][0]['chunk_id'])
        self.req('POST',f'/knowledge/{doc}/reindex',{})
        stale=self.req('POST',f"/tickets/{t['id']}/messages",{'body':draft['draft'],'proposal_id':draft['id'],'idempotency_key':key()})
        self.assertEqual(stale.status_code,400)
        self.assertEqual(self.req('DELETE',f'/knowledge/{doc}').status_code,200)
        self.assertEqual(self.req('GET',f'/knowledge/{doc}/chunks').status_code,404)
    def test_02_event_priority_and_durable_failure(self):
        wid=self.workflow('ticket.created','priority:high')
        t=self.ticket('Event priority');self.assertEqual(t['priority'],'high')
        runs=self.req('GET','/workflow-runs').json();self.assertTrue(any(r['workflow_id']==wid and r['trigger_event']=='ticket.created' and r['state']=='completed' for r in runs))
        definition={'nodes':[{'id':'start','kind':'trigger','next':'tag'},{'id':'tag','kind':'action','value':'label:must-rollback','next':'bad'},{'id':'bad','kind':'action','value':'status:closed'}]}
        w=self.req('POST','/workflows',{'name':'Synthetic failure','definition':definition}).json()
        self.req('POST',f"/workflows/{w['id']}/publish",{})
        r=self.req('POST',f"/workflows/{w['id']}/run",{'ticket_id':t['id'],'simulation':False,'idempotency_key':key()})
        self.assertEqual(r.status_code,200,r.text);self.assertEqual(r.json()['state'],'failed');self.assertEqual(r.json()['trace'][-1]['kind'],'error')
        self.assertNotIn('must-rollback',self.req('GET',f"/tickets/{t['id']}").json()['labels'])
        self.assertTrue(any(run['id']==r.json()['id'] for run in self.req('GET','/workflow-runs').json()))
    def test_03_reply_reopens_and_resolved_requests_feedback(self):
        self.workflow('customer.replied','status:open');self.workflow('ticket.resolved','csat:request')
        t=self.ticket('Reply lifecycle');tid=t['id']
        self.assertEqual(self.req('PATCH',f'/tickets/{tid}',{'status':'solved'}).status_code,200)
        messages=self.req('GET',f'/tickets/{tid}/messages',headers=self.ch).json();self.assertTrue(any('How did we do?' in m['body'] for m in messages))
        r=self.req('POST',f'/tickets/{tid}/messages',{'body':'Still need help','idempotency_key':key()},self.ch);self.assertEqual(r.status_code,200,r.text)
        self.assertEqual(self.req('GET',f'/tickets/{tid}').json()['status'],'open')
    def test_04_abstention_human_approval_and_ai_activity(self):
        self.workflow('ai.abstained','approval');t=self.ticket('noevidence-spacecraft-718')
        draft=self.req('POST',f"/tickets/{t['id']}/grounded-draft",{});self.assertTrue(draft.json()['abstained'])
        approvals=self.req('GET','/approvals').json();review=next(a for a in approvals if a['action']=='human.review' and a['ticket_id']==t['id'])
        decided=self.req('POST',f"/approvals/{review['id']}",{'approve':True});self.assertEqual(decided.json()['state'],'executed')
        logs=self.req('GET','/ai/activity');self.assertEqual(logs.status_code,200);self.assertTrue(any(l['status']=='abstained' for l in logs.json()))
        self.assertEqual(self.req('GET','/ai/activity',headers=self.ch).status_code,403)
        self.assertEqual(self.req('GET','/ai/activity',headers=self.bh).json(),[])
    def test_05_pagination_and_tenant_filter(self):
        for i in range(8):self.ticket(f'Page fixture {i}')
        first=self.req('GET','/inbox?limit=3&offset=0&q=Page%20fixture').json();second=self.req('GET','/inbox?limit=3&offset=3&q=Page%20fixture').json()
        self.assertEqual(len(first),3);self.assertEqual(len(second),3);self.assertFalse({r['id'] for r in first}&{r['id'] for r in second})
        self.assertEqual(self.req('GET','/inbox?limit=3&q=Page%20fixture',headers=self.bh).json(),[])
    def test_06_sleep_safe_notification_projection(self):
        t=self.ticket('Notify during request')
        notes=self.req('GET','/notifications').json();ids={n['id'] for n in notes}
        self.assertTrue(any(n['resource_id']==t['id'] for n in notes))
        self.assertEqual(ids,{n['id'] for n in self.req('GET','/notifications').json()})
    def test_07_ai_failure_does_not_block_manual_support(self):
        t=self.ticket('Service failure fallback')
        for suffix in ['/summary','/category']:
            r=self.req('GET',f"/tickets/{t['id']}{suffix}");self.assertEqual(r.status_code,200,r.text)
        reply=self.req('POST',f"/tickets/{t['id']}/messages",{'body':'Manual answer remains available','idempotency_key':key()});self.assertEqual(reply.status_code,200)
        if os.getenv('EXPECT_AI_FALLBACK')=='true':
            logs=self.req('GET','/ai/activity').json();self.assertTrue(any(l['ticket_id']==t['id'] and l['status']=='fallback' for l in logs))
if __name__=='__main__':unittest.main(verbosity=2)
