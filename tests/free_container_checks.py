"""Verify the actual combined beta image under the local free-instance limits."""
import json
import pathlib
import subprocess
import time
import uuid
import httpx

ROOT = pathlib.Path(__file__).resolve().parents[1]
CMD = ['docker','compose','-f','compose.stage1.yaml','-p','aster-stage1-free-verify']
def compose(*args):
    return subprocess.check_output(CMD+list(args),cwd=ROOT,text=True).strip()
def inspect(container, template):
    return subprocess.check_output(['docker','inspect','--format',template,container],text=True).strip()

def main():
    base='http://localhost:18083'
    client=httpx.Client(base_url=base,headers={'Origin':'http://localhost:3000'},timeout=30)
    assert client.get('/release').json()['synthetic_emails_required'] is True
    payload={'email':f'profile-{uuid.uuid4()}@example.test','password':'Synthetic-beta-container-password!',
             'organization_name':'Synthetic constrained runtime','full_name':'Synthetic tester'}
    assert client.post('/auth/signup',json=payload|{'email':'real@example.com'}).status_code==400
    signup=client.post('/auth/signup',json=payload);signup.raise_for_status()
    headers={'Authorization':'Bearer '+signup.json()['token']}
    client.cookies.clear()
    assert client.get('/sandbox/mailbox',headers=headers).status_code==403
    assert client.post('/invitations',headers=headers,json={'email':'real@example.com','role':'customer'}).status_code==400
    assert client.get('/billing',headers=headers).json()['charges_enabled'] is False
    ticket=client.post('/tickets',headers=headers,json={'subject':'Combined supervisor persistence','description':'Synthetic only'})
    ticket.raise_for_status(); path='/tickets/'+ticket.json()['id']
    container=compose('ps','-q','backend')
    assert int(inspect(container,'{{.HostConfig.Memory}}'))==512*1024*1024
    assert int(inspect(container,'{{.HostConfig.NanoCpus}}'))==100_000_000
    assert compose('exec','-T','backend','id','-u')!='0'
    assert compose('exec','-T','backend','python','-c',"import os;assert os.getenv('BETA_MODE')=='true';assert os.getenv('LLM_ENABLED')=='false';from app.llm_client import LLMClient;assert not LLMClient().is_configured") == ''
    memory_before_crash=json.loads(compose('exec','-T','backend','python','-c',"import pathlib,json;print(json.dumps({n:pathlib.Path('/sys/fs/cgroup/'+n).read_text().strip() for n in ['memory.current','memory.peak','memory.max']}))"))
    before=int(inspect(container,'{{.RestartCount}}'))
    started=time.monotonic()
    # Kill only the owned internal Uvicorn worker, not PID 1 or other containers.
    compose('exec','-T','backend','python','-c',"import pathlib,os,signal; workers=[int(p.parent.name) for p in pathlib.Path('/proc').glob('[0-9]*/cmdline') if p.read_bytes().split(b'\\0')[1:3]==[b'-m',b'uvicorn']];assert len(workers)==1;os.kill(workers[0],signal.SIGTERM)")
    for _ in range(180):
        try:
            if int(inspect(container,'{{.RestartCount}}'))>before and client.get('/ready').status_code==200:
                break
        except (httpx.HTTPError,subprocess.CalledProcessError):
            pass
        time.sleep(1)
    else:raise AssertionError('Combined supervisor did not recover after AI worker exit')
    recovery_ms=round((time.monotonic()-started)*1000)
    assert client.get(path,headers=headers).status_code==200
    assert inspect(container,'{{.State.OOMKilled}}')=='false'
    memory=json.loads(compose('exec','-T','backend','python','-c',"import pathlib,json;print(json.dumps({n:pathlib.Path('/sys/fs/cgroup/'+n).read_text().strip() for n in ['memory.current','memory.peak','memory.max']}))"))
    report={'environment':'Local Docker Desktop Linux, actual combined release image; not hosted Render',
        'memory_limit_bytes':512*1024*1024,'cpu_limit':0.1,'cgroup_memory_bytes':memory,
        'cgroup_memory_before_worker_crash':memory_before_crash,
        'beta_guards':'passed: real emails and sandbox denied, billing and model inference disabled',
        'non_root':'passed','ai_worker_crash_supervisor_restart':'passed','worker_crash_recovery_ms':recovery_ms,
        'persisted_ticket_after_restart':'passed','oom_killed':False,
        'limits':'Small synthetic tests only; no hosted cold start, sustained load or uptime guarantee'}
    (ROOT/'docs/STAGE1_FREE_CONTAINER_CHECKS.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

if __name__=='__main__': main()
