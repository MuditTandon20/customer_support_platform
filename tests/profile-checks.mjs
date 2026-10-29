import pg from 'pg';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {randomUUID} from 'node:crypto';
import {resolve} from 'node:path';
import {writeFile} from 'node:fs/promises';
const root=resolve('..'),windows=process.platform==='win32';
const python=resolve(root,windows?'backend/ai-copilot/.venv/Scripts/python.exe':'backend/ai-copilot/.venv/bin/python');
const client=new pg.Client({connectionString:'postgres://postgres:local-test-only@127.0.0.1:55433/postgres'});await client.connect();
async function database(){const name='stage1_profile_'+randomUUID().replaceAll('-','');await client.query(`CREATE DATABASE "${name}"`);return name;}
const shared={...process.env,JWT_SECRET:'profile-synthetic-secret-at-least-32-characters',AI_SERVICE_TOKEN:'profile-synthetic-service-secret-32-characters',FRONTEND_ORIGIN:'http://localhost:13000',BACKGROUND_PROCESSING:'false',ENABLE_SANDBOX:'false',BETA_MODE:'false',LLM_ENABLED:'false',AI_COPILOT_URL:'http://127.0.0.1:19999'};
delete shared.KAFKA_BROKERS;
async function start(name,port,extra={}){const env={...shared,DATABASE_URL:`postgres://postgres:local-test-only@127.0.0.1:55433/${name}`,API_BIND:`127.0.0.1:${port}`,...extra};const child=spawn(resolve(root,'backend/rust-api/target/debug/rust-api'+(windows?'.exe':'')),[],{cwd:root,env,stdio:'ignore',windowsHide:true});for(let i=0;i<100;i++){try{if((await fetch(`http://localhost:${port}/ready`)).ok)return {child,env};}catch{}await new Promise(r=>setTimeout(r,100));}child.kill();throw new Error('Profile failed to start');}
async function stop(child){if(child.exitCode===null){const exit=once(child,'exit');child.kill();await exit;}}
async function post(port,path,data,token){const r=await fetch(`http://localhost:${port}${path}`,{method:'POST',headers:{'Content-Type':'application/json',Origin:shared.FRONTEND_ORIGIN,...(token?{Authorization:`Bearer ${token}`}:{})},body:JSON.stringify(data)});return {status:r.status,body:await r.json()};}
const signup=email=>({email,password:'Profile-test-password-2026!',organization_name:'Synthetic profile fixture',full_name:'Synthetic tester'});
let fault,beta;
try{
 const faultDb=await database();fault=await start(faultDb,18081);
 const code=await new Promise((ok,fail)=>{const test=spawn(python,['tests/stage1_integration.py'],{cwd:root,env:{...fault.env,TEST_API_URL:'http://localhost:18081',EXPECT_AI_FALLBACK:'true'},stdio:'inherit',windowsHide:true});test.on('error',fail);test.on('exit',ok);});
 if(code!==0)throw new Error('Service-failure regression failed');
 const account=await post(18081,'/auth/signup',signup(`restart-${randomUUID()}@example.test`));
 const ticket=await post(18081,'/tickets',{subject:'Synthetic restart persistence'},account.body.token);
 await stop(fault.child);fault=await start(faultDb,18081);
 const persisted=await fetch(`http://localhost:18081/tickets/${ticket.body.id}`,{headers:{Authorization:`Bearer ${account.body.token}`}});if(persisted.status!==200)throw new Error('Restart lost durable state');
 beta=await start(await database(),18082,{BETA_MODE:'true',MAX_ORGANIZATIONS:'1'});
 if((await post(18082,'/auth/signup',signup('real@example.com'))).status!==400)throw new Error('Real email accepted by beta');
 const candidates=await Promise.all(['first','second'].map(name=>post(18082,'/auth/signup',signup(`${name}@example.test`))));
 if(candidates.filter(r=>r.status===200).length!==1||candidates.filter(r=>r.status===400).length!==1)throw new Error('Organization cap race');
 const token=candidates.find(r=>r.status===200).body.token;
 if((await post(18082,'/invitations',{email:'real@example.com',role:'customer'},token)).status!==400)throw new Error('Real invitation accepted');
 const mailbox=await fetch('http://localhost:18082/sandbox/mailbox',{headers:{Authorization:`Bearer ${token}`}});if(mailbox.status!==403)throw new Error('Public sandbox exposed');
 await writeFile('../docs/STAGE1_PROFILE_CHECKS.json',JSON.stringify({ai_service_failure:'passed: seven API suites with unavailable service and explicit fallback assertions',native_restart:'passed: owned ticket accessible after process restart with same Postgres',public_beta:'passed: synthetic emails, concurrent organization cap and disabled sandbox',container_restart:'Not exercised by this native script; see STAGE1_CONTAINER_CHECKS.json for separate Docker verification'},null,2));
 console.log('Fault, restart and public-beta profile checks passed.');
}finally{if(fault)await stop(fault.child);if(beta)await stop(beta.child);await client.end();}
