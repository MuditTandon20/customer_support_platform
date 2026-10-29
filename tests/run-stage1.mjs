import EmbeddedPostgres from 'embedded-postgres';
import {spawn,spawnSync} from 'node:child_process';
import {mkdtemp,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {resolve,join} from 'node:path';
const root=resolve('..'),windows=process.platform==='win32';
const directory=await mkdtemp(join(tmpdir(),'aster-stage1-'));
const db=new EmbeddedPostgres({databaseDir:directory,user:'postgres',password:'local-test-only',port:55433,persistent:true,onLog:()=>{},onError:m=>process.stderr.write(String(m)+'\n')});
await db.initialise();await db.start();await db.createDatabase('support_os_test');
const python=resolve(root,windows?'backend/ai-copilot/.venv/Scripts/python.exe':'backend/ai-copilot/.venv/bin/python');
const env={...process.env,DATABASE_URL:'postgres://postgres:local-test-only@127.0.0.1:55433/support_os_test',JWT_SECRET:'stage1-isolated-test-secret-at-least-32-characters',AI_SERVICE_TOKEN:'stage1-isolated-service-token',API_BIND:'127.0.0.1:18080',FRONTEND_ORIGIN:'http://localhost:13000',AI_COPILOT_URL:'http://127.0.0.1:18001',API_PROXY_URL:'http://127.0.0.1:18080',ENABLE_SANDBOX:'true',BETA_MODE:'false',BACKGROUND_PROCESSING:'false',LLM_ENABLED:'false'};
delete env.KAFKA_BROKERS;
const start=Date.now();
const children=[
 spawn(python,['-m','uvicorn','app.main:app','--host','127.0.0.1','--port','18001'],{cwd:resolve(root,'backend/ai-copilot'),env,stdio:'inherit',windowsHide:true}),
 spawn(resolve(root,'backend/rust-api/target/debug/rust-api'+(windows?'.exe':'')),[],{cwd:root,env,stdio:'inherit',windowsHide:true}),
 spawn(process.execPath,[resolve(root,'frontend/node_modules/next/dist/bin/next'),'start','--port','13000'],{cwd:resolve(root,'frontend'),env,stdio:'inherit',windowsHide:true})
];
async function ready(url){for(let i=0;i<150;i++){if(children.some(c=>c.exitCode!==null))throw new Error('A test service exited');try{if((await fetch(url)).ok)return;}catch{}await new Promise(r=>setTimeout(r,200));}throw new Error('Not ready: '+url);}
const stop=async()=>{for(const c of children){if(windows)spawnSync("taskkill",["/PID",String(c.pid),"/T","/F"],{windowsHide:true,stdio:"ignore"});else c.kill();}await db.stop();process.exit(0);};
process.on('SIGINT',stop);process.on('SIGTERM',stop);
try{await ready('http://localhost:18080/ready');await ready('http://localhost:18001/health');await ready('http://localhost:13000/api/health');
 await writeFile(resolve(root,'docs/stage1-local-runtime.json'),JSON.stringify({environment:'Windows/native debug Rust, isolated Python, production Next.js, disposable Postgres 18',ready_ms:Date.now()-start,pids:{python:children[0].pid,rust:children[1].pid,frontend:children[2].pid},background_polling:false,database_directory:directory},null,2));
 console.log('STAGE1 STACK READY: http://localhost:13000; no existing database changed.');
}catch(e){console.error(e);await stop();}
setInterval(()=>{},30000);
