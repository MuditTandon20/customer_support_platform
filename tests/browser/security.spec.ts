import {test, expect} from '@playwright/test';

test('conversation treats hostile HTML as text and proxy cannot replace its upstream host', async ({page, request}) => {
  const api = process.env.TEST_API_URL || 'http://localhost:18080';
  const origin = process.env.TEST_WEB_URL || 'http://localhost:13000';
  const email = `xss-${Date.now()}@example.test`, password = 'Synthetic-security-test-password!';
  const signup = await request.post(api+'/auth/signup', {headers:{Origin:origin},data:{email,password,organization_name:'Synthetic security fixture',full_name:'Synthetic reviewer'}});
  expect(signup.ok()).toBeTruthy();
  const auth = {Origin:origin,Authorization:'Bearer '+(await signup.json()).token};
  const created = await request.post(api+'/tickets',{headers:auth,data:{subject:'Hostile content fixture',description:'Synthetic content safety check'}});
  expect(created.ok()).toBeTruthy();
  const ticket = await created.json();
  const payload = '<img src=x onerror="window.__asterXss=1"><script>window.__asterXss=1</script>';
  const message = await request.post(api+`/tickets/${ticket.id}/messages`,{headers:auth,data:{body:payload,idempotency_key:crypto.randomUUID()}});
  expect(message.ok()).toBeTruthy();
  await page.goto('/login');
  await page.getByLabel('Email address').fill(email);
  await page.getByLabel('Password',{exact:true}).fill(password);
  await page.getByRole('button',{name:'Sign in',exact:false}).click();
  await page.waitForURL('**/workspace/dashboard');
  await page.goto(`/tickets/${ticket.id}`);
  await expect(page.getByText(payload,{exact:true})).toBeVisible();
  expect(await page.evaluate(()=>('__asterXss' in window))).toBeFalsy();
  // Absolute-looking path segments remain paths on the configured API host.
  const escaped = await request.get(origin+'/api/https%3A%2F%2Fexample.invalid/probe');
  expect(escaped.status()).toBe(404);
});
