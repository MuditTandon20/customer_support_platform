import {test,expect} from '@playwright/test';

test('sign-in uses the website proxy and retains its HttpOnly session after refresh',async({page,request,context})=>{
 const api=process.env.TEST_API_URL||'http://localhost:18080';
 const web=process.env.TEST_WEB_URL||'http://localhost:13000';
 const email=`connection-${Date.now()}@example.test`,password='Connection-test-password-2026!';
 const signup=await request.post(api+'/auth/signup',{headers:{Origin:web},data:{email,password,organization_name:'Connection regression fixture',full_name:'Connection fixture'}});
 expect(signup.ok()).toBeTruthy();
 const directRequests:string[]=[];
 page.on('request',r=>{if(new URL(r.url()).origin===new URL(api).origin)directRequests.push(r.url());});
 await page.goto('/login');
 await page.getByLabel('Email address').fill(email);
 await page.getByLabel('Password',{exact:true}).fill(password);
 const response=page.waitForResponse(r=>new URL(r.url()).pathname==='/api/auth/login');
 await page.getByRole('button',{name:'Sign in',exact:false}).click();
 expect((await response).status()).toBe(200);
 await expect(page.locator('.org-switch')).toContainText('Connection regression fixture');
 expect(directRequests).toEqual([]);
 expect((await context.cookies()).find(c=>c.name==='aster_session')?.httpOnly).toBe(true);
 await page.reload();
 await expect(page.locator('.org-switch')).toContainText('Connection regression fixture');
});
