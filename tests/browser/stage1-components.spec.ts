import {test,expect} from '@playwright/test';

test('auth component preserves input and recovers from a network error',async({page})=>{
 await page.goto('/login');
 await page.route('**/api/auth/login',route=>route.abort('connectionrefused'));
 await page.getByLabel('Email address').fill('synthetic-component@example.test');
 await page.getByLabel('Password',{exact:true}).fill('Synthetic-component-password!');
 await page.getByRole('button',{name:'Sign in',exact:false}).click();
 await expect(page.locator('.error-banner')).toContainText('Cannot reach the support service.');
 await expect(page.getByLabel('Email address')).toHaveValue('synthetic-component@example.test');
 await expect(page.getByRole('button',{name:'Sign in',exact:false})).toBeEnabled();
 await expect(page.getByRole('note')).toContainText('TESTING BETA');
});

test('workflow component supports drag, keyboard reorder and a persisted trigger template',async({page,request})=>{
 const api=process.env.TEST_API_URL||'http://localhost:18080',origin=process.env.TEST_WEB_URL||'http://localhost:13000';
 const email=`studio-component-${Date.now()}@example.test`,password='Synthetic-component-password!';
 const created=await request.post(api+'/auth/signup',{headers:{Origin:origin},data:{email,password,organization_name:'Synthetic component fixture',full_name:'Synthetic tester'}});expect(created.ok()).toBeTruthy();
 await page.goto('/login');await page.getByLabel('Email address').fill(email);await page.getByLabel('Password',{exact:true}).fill(password);await page.getByRole('button',{name:'Sign in',exact:false}).click();
 await expect(page.locator('.org-switch')).toBeVisible();await page.goto('/workspace/workflows');
 const nodes=page.locator('.workflow-node');await expect(nodes).toHaveCount(3);
 await nodes.filter({hasText:'Human review'}).dragTo(nodes.filter({hasText:'label:reviewed'}));
 await expect(nodes.nth(1)).toContainText('Human review');
 await nodes.nth(1).click();await page.getByRole('button',{name:'↓ Move',exact:true}).click();
 await expect(nodes.nth(2)).toContainText('Human review');
 await page.getByRole('button',{name:'Prioritize new tickets',exact:true}).click();
 await expect(nodes).toHaveCount(2);await expect(nodes.first()).toContainText('ticket.created');
 const saved=page.waitForResponse(r=>new URL(r.url()).pathname==='/api/workflows'&&r.request().method()==='POST');
 await page.getByRole('button',{name:'Save draft',exact:true}).click();expect((await saved).ok()).toBeTruthy();
 await page.getByRole('button',{name:'Publish version'}).click();await expect(page.getByText('Version 1 published. Execution uses this saved version.')).toBeVisible();
 await page.screenshot({path:'../docs/screenshots/stage1-workflow.png',fullPage:true});
});
