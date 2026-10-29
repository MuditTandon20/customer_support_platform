import {test,expect} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
test('customer portal hides internal notes, receives a cited reply, and submits CSAT',async({page,request})=>{
 const api=process.env.TEST_API_URL||'http://localhost:18080';const origin=process.env.TEST_WEB_URL||'http://localhost:13000';const suffix=Date.now();
 async function post(path:string,data:unknown,token?:string){const r=await request.post(api+path,{data,headers:{Origin:origin,...(token?{Authorization:`Bearer ${token}`}:{})}});expect(r.ok(),await r.text()).toBeTruthy();return r.json();}
 const admin=await post('/auth/signup',{organization_name:`Demo customer QA ${suffix}`,full_name:'Demo team',email:`portal-admin-${suffix}@example.test`,password:'Portal-password-2026!'});
 const email=`portal-customer-${suffix}@example.test`,password='Portal-password-2026!';
 const invite=await post('/invitations',{email,role:'customer'},admin.token);
 const customer=await post('/auth/accept-invitation',{email,password,token:invite.token,full_name:'Demo customer'});
 const ticket=await post('/tickets',{subject:'Refund policy',description:'Refund policy'},customer.token);
 await post(`/tickets/${ticket.id}/messages`,{body:'Internal-only account review',internal:true,idempotency_key:crypto.randomUUID()},admin.token);
 await post('/knowledge',{title:'Refund policy',content:'Refund requests are reviewed within five business days.',visibility:'public'},admin.token);
 const proposal=await post(`/tickets/${ticket.id}/grounded-draft`,{},admin.token);
 await post(`/tickets/${ticket.id}/messages`,{body:proposal.draft,proposal_id:proposal.id,internal:false,idempotency_key:crypto.randomUUID()},admin.token);
 await page.goto('/login');await page.getByLabel('Email address').fill(email);await page.getByLabel('Password',{exact:true}).fill(password);await page.getByRole('button',{name:'Sign in',exact:false}).click();
 await page.waitForURL('**/workspace/dashboard');
 await page.goto(`/tickets/${ticket.id}`);
 await expect(page.getByRole('heading',{name:'Refund policy',exact:true})).toBeVisible();
 await expect(page.getByText('Refund requests are reviewed within five business days.',{exact:false})).toBeVisible();
 await expect(page.getByText('Internal-only account review')).toHaveCount(0);
 await expect(page.getByRole('button',{name:'Internal note',exact:true})).toHaveCount(0);
 await page.getByLabel('Your reply',{exact:true}).fill('Thank you for the clarification.');await page.getByRole('button',{name:'Send portal reply'}).click();
 await expect(page.getByText('Thank you for the clarification.',{exact:true})).toBeVisible();
 const updated=await request.patch(`${api}/tickets/${ticket.id}`,{data:{status:'solved'},headers:{Origin:origin,Authorization:`Bearer ${admin.token}`}});expect(updated.ok()).toBeTruthy();
 await page.reload();await page.getByRole('button',{name:'5 ★',exact:true}).click();await expect(page.getByText('Thank you. Your feedback was saved.')).toBeVisible();
 const a11y=await new AxeBuilder({page}).withTags(['wcag2a','wcag2aa','wcag21aa']).analyze();expect(a11y.violations.filter(v=>v.impact==='critical'||v.impact==='serious')).toEqual([]);
 await page.screenshot({path:'../docs/screenshots/customer-portal.png',fullPage:true});
 await page.goto('/widget');await expect(page.getByRole('link',{name:'Open your conversations'})).toBeVisible();
});
