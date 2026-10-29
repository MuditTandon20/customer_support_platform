import pg from 'pg';
import {spawn} from 'node:child_process';
import {randomUUID} from 'node:crypto';
import {resolve} from 'node:path';
import {writeFile} from 'node:fs/promises';
const root=resolve('..');
const database='support_os_fresh_'+randomUUID().replaceAll('-','');
const client=new pg.Client({connectionString:'postgres://postgres:local-test-only@127.0.0.1:55432/postgres'});
await client.connect();await client.query(`CREATE DATABASE "${database}"`);await client.end();
const env={...process.env,DATABASE_URL:`postgres://postgres:local-test-only@127.0.0.1:55432/${database}`,JWT_SECRET:'fresh-db-test-secret-at-least-32-characters',API_BIND:'127.0.0.1:18081',FRONTEND_ORIGIN:'http://localhost:13000',AI_SERVICE_TOKEN:'test-service-token',AI_COPILOT_URL:'http://127.0.0.1:18001',ENABLE_SANDBOX:'true'};
delete env.KAFKA_BROKERS;
const windows=process.platform==='win32';
const api=spawn(resolve(root,'backend/rust-api/target/debug/rust-api'+(windows?'.exe':'')),[],{cwd:root,env,stdio:'ignore',windowsHide:true});
try{
 let ready=false;for(let i=0;i<100;i++){try{ready=(await fetch('http://localhost:18081/health')).ok;}catch{}if(ready)break;await new Promise(r=>setTimeout(r,100));}
 if(!ready)throw new Error('Fresh API failed to migrate/start');
 const python=resolve(root,windows?'backend/ai-copilot/.venv/Scripts/python.exe':'backend/ai-copilot/.venv/bin/python');
 const code=await new Promise((ok,fail)=>{const test=spawn(python,['tests/integration.py'],{cwd:root,env:{...env,TEST_API_URL:'http://localhost:18081'},stdio:'inherit',windowsHide:true});test.on('error',fail);test.on('exit',ok);});
 if(code!==0)throw new Error('Fresh-stack integration failed');
 await writeFile(resolve(root,'docs/fresh-migration-verification.json'),JSON.stringify({status:'passed',database,scope:'Fresh empty PostgreSQL 18 database; all additive migrations; latest compiled Rust; full HTTP integration suite',data_policy:'New test database retained for inspection; no existing database reset'},null,2));
 console.log('Fresh-database migration and integration verification passed.');
}finally{api.kill();}
