import {test,expect} from '@playwright/test';

test('duplicate signup explains existing account and links to a working sign-in',async({page,request})=>{
 const email=`signup-conflict-${Date.now()}@example.test`,password='Conflict-test-password-2026!';
 const response=await request.post((process.env.TEST_API_URL||'http://localhost:18080')+'/auth/signup',{headers:{Origin:process.env.TEST_WEB_URL||'http://localhost:13000'},data:{email,password,organization_name:'Original signup fixture',full_name:'Signup fixture'}});
 expect(response.ok()).toBeTruthy();
 await page.goto('/signup');
 await page.getByLabel('Organization name').fill('A different organization');
 await page.getByLabel('Full name',{exact:true}).fill('Signup fixture');
 await page.getByLabel('Email address').fill(email);
 await page.getByLabel('Password',{exact:true}).fill(password);
 await page.getByRole('button',{name:'Create workspace'}).click();
 await expect(page.locator('.error-banner[role=alert]')).toContainText('An account with this email already exists.');
 await expect(page.locator('.error-banner[role=alert]')).not.toContainText('idempotency');
 await page.getByRole('link',{name:'Sign in to your existing account'}).click();
 await expect(page).toHaveURL(/\/login$/);
 await page.getByLabel('Email address').fill(email);
 await page.getByLabel('Password',{exact:true}).fill(password);
 await page.getByRole('button',{name:'Sign in',exact:false}).click();
 await expect(page.locator('.org-switch')).toContainText('Original signup fixture');
});
