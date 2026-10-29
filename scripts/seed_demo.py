"""Create a clearly labeled, local-only demo. Does not modify an existing organization."""
import os
import uuid
from urllib.parse import urlparse
import httpx

base=os.getenv('DEMO_API_URL','http://localhost:8080')
if urlparse(base).hostname not in {'localhost','127.0.0.1'}:
    raise SystemExit('Demo seeding is restricted to localhost. Use a local stack.')
suffix=uuid.uuid4().hex[:6]
email=f'demo-admin-{suffix}@example.test'
password='Demo-only-password-2026!'
client=httpx.Client(base_url=base,timeout=30,headers={'Origin':os.getenv('DEMO_FRONTEND_ORIGIN','http://localhost:3000')})
def request(method,path,payload=None,token=None):
    r=client.request(method,path,json=payload,headers={'Authorization':f'Bearer {token}'} if token else {})
    r.raise_for_status()
    return r.json()
admin=request('POST','/auth/signup',{'organization_name':f'Demo · Acme {suffix}','email':email,'password':password,'full_name':'Demo · Alex Morgan'})
token=admin['token']
customers=[]
for name in ['Jamie Rivera','Sam Chen','Taylor Brooks']:
    customer_email=f'demo-{name.split()[0].lower()}-{suffix}@example.test'
    invitation=request('POST','/invitations',{'email':customer_email,'role':'customer'},token)
    customer=request('POST','/auth/accept-invitation',{'token':invitation['token'],'email':customer_email,'password':password,'full_name':f'Demo · {name}'})
    customers.append(customer)
for title,content,visibility in [
 ('Demo · Refund policy','Refund requests are reviewed by the support team within five business days. No refund is automatic. Include the invoice number in your request.','public'),
 ('Demo · Reset your password','To reset your password, open the sign-in page and choose Reset your password. In local sandbox mode, the reset link appears in the administrator sandbox mailbox.','public'),
 ('Demo · API troubleshooting','For API errors, collect the request ID, timestamp and response status. Never ask customers to share API secrets. Escalate production outages to the responsible team.','team')]:
    request('POST','/knowledge',{'title':title,'content':content,'visibility':visibility},token)
subjects=['Refund policy','Reset password help','API error investigation','Invoice review','Welcome and onboarding','Subscription cancellation review']
tickets=[]
for i,subject in enumerate(subjects):
    t=request('POST','/tickets',{'subject':f'Demo · {subject}','description':f'Demo fixture: I need help with {subject.lower()}. This is synthetic customer data.'},customers[i%len(customers)]['token'])
    request('PATCH',f"/tickets/{t['id']}/metadata",{'priority':['normal','low','urgent'][i%3],'labels':['demo',['billing','account','technical'][i%3]]},token)
    if i%2==0:request('PATCH',f"/tickets/{t['id']}",{'assigned_to':admin['user_id'],'status':'open'},token)
    tickets.append(t)
request('POST',f"/tickets/{tickets[0]['id']}/grounded-draft",{},token)
request('POST',f"/tickets/{tickets[0]['id']}/messages",{'body':'Demo reply: we are reviewing your request. No real refund has been issued.','internal':False,'idempotency_key':str(uuid.uuid4())},token)
request('POST','/tools',{'ticket_id':tickets[5]['id'],'action':'subscription.cancel','arguments':{'reason':'Demo approval fixture. No external subscription exists.'},'idempotency_key':str(uuid.uuid4())},token)
workflow=request('POST','/workflows',{'name':'Demo · Review before action','definition':{'nodes':[{'id':'start','kind':'trigger','next':'tag'},{'id':'tag','kind':'action','value':'label:needs-review','next':'review'},{'id':'review','kind':'approval','value':'Human handoff'}]}},token)
request('POST',f"/workflows/{workflow['id']}/publish",{},token)
print(f'Created LOCAL DEMO workspace.\nEmail: {email}\nPassword: {password}\nAll fixtures are explicitly labeled Demo. No email or payment was sent.')
