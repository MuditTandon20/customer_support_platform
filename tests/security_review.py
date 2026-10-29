"""Exercise the runtime boundaries used in the Stage 1 CVE assessment.

Only the allowlisted local rehearsal projects are inspected or written to.
No scanner findings are suppressed by this test.
"""
import json
import ast
import os
from pathlib import Path
import platform
import re
import subprocess
import time
import uuid
import httpx
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
from review_security import digest_sources

ROOT=Path(__file__).resolve().parents[1]
PREFIX=os.getenv('ASTER_VERIFY_PROJECT_PREFIX','aster-stage1')
assert PREFIX in {'aster-stage1','aster-stage1-security'}
FULL=['docker','compose','-p',PREFIX+'-verify']
BETA=['docker','compose','-f','compose.stage1.yaml','-p',PREFIX+'-free-verify']
def command(*args):return subprocess.check_output(list(args),cwd=ROOT,text=True).strip()
def main():
    records=[]
    for compose,services in [(FULL,['db','api','ai','web']),(BETA,['backend'])]:
        for service in services:
            cid=command(*compose,'ps','-q',service)
            info=json.loads(command('docker','inspect',cid))[0]
            config=info['HostConfig']
            assert config['ReadonlyRootfs'] is True
            assert 'ALL' in config['CapDrop']
            assert any(x.startswith('no-new-privileges') for x in config['SecurityOpt'])
            assert config['Privileged'] is False and config.get('PidMode')!='host'
            uid=command(*compose,'exec','-T',service,'id','-u');assert uid!='0'
            status=command(*compose,'exec','-T',service,'cat','/proc/1/status')
            assert re.search(r'CapEff:\s+0+\s',status)
            assert re.search(r'NoNewPrivs:\s+1',status)
            # Inspect image files independently of the persistent DB volume.
            # This isolated read-only inspector needs DAC_OVERRIDE to enumerate
            # private _apt cache directories; application processes retain no caps.
            image=info['Image']
            suid=subprocess.run(['docker','run','--rm','--read-only','--network','none',
                '--cap-drop','ALL','--cap-add','DAC_OVERRIDE','--security-opt','no-new-privileges','--user','0',
                '--entrypoint','sh',image,'-c','find / -xdev -type f -perm /6000 -print'],
                capture_output=True,text=True,check=True)
            assert not suid.stdout.strip(),suid.stdout
            unavailable=command(*compose,'exec','-T',service,'sh','-c',
                r'find /usr -type f \( -name systemd-homed -o -name tpm2daemon \) -print; for p in /dev/tpm0 /dev/tpmrm0; do if test -e "$p"; then echo "$p"; fi; done')
            assert not unavailable,unavailable
            if service=='web':command(*compose,'exec','-T',service,'sh','-c','test -w /app/.next/cache')
            if service in ['db','ai']:
                assert not any(info['NetworkSettings']['Ports'].values())
            records.append({'service':service,'image_id':image,'uid':int(uid),'capabilities':'none',
                'no_new_privileges':True,'rootfs_read_only':True,'suid_sgid_files':[],
                'homed_tpm_daemon_or_device':[]})
    # Health probe has no dependency on credentials and cannot report a dead port healthy.
    command(*FULL,'exec','-T','api','env','-u','DATABASE_URL','-u','JWT_SECRET','rust-api','--healthcheck')
    failed=subprocess.run(FULL+['exec','-T','api','env','API_BIND=127.0.0.1:9','rust-api','--healthcheck'],capture_output=True)
    assert failed.returncode!=0
    base=os.getenv('TEST_API_URL','http://localhost:8080')
    client=httpx.Client(base_url=base,headers={'Origin':os.getenv('TEST_WEB_URL','http://localhost:3000')},timeout=30)
    # Earlier abuse tests intentionally exhaust the shared 60-second auth bucket.
    # Respect the production limit, retry only 429, and fail after one window.
    credentials={'email':f'boundary-{uuid.uuid4()}@example.test',
        'password':'Synthetic-boundary-test-password!','organization_name':'Synthetic security boundaries','full_name':'Synthetic reviewer'}
    deadline=time.monotonic()+65
    while True:
        signup=client.post('/auth/signup',json=credentials)
        if signup.status_code!=429 or time.monotonic()>=deadline:break
        time.sleep(min(5,max(0,deadline-time.monotonic())))
    signup.raise_for_status();auth={'Authorization':'Bearer '+signup.json()['token']};client.cookies.clear()
    xml='<!DOCTYPE note [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><note>&xxe;</note>'
    assert client.post('/tickets',headers=auth|{'Content-Type':'application/xml'},content=xml).status_code==415
    ticket=client.post('/tickets',headers=auth,json={'subject':'Synthetic XML is text','description':xml})
    ticket.raise_for_status();tid=ticket.json()['id']
    assert client.get('/tickets/'+tid,headers=auth).json()['description']==xml
    doc=client.post('/knowledge',headers=auth,json={'title':'Synthetic XML text','content':xml,'visibility':'team'})
    doc.raise_for_status();assert doc.json()['content']==xml
    assert client.post('/tools',headers=auth,json={'ticket_id':tid,'action':'shell.exec','arguments':{'command':'id'},'idempotency_key':str(uuid.uuid4())}).status_code in (400,422)
    # Confirm the storage boundary independently of the HTTP response.
    sql="SELECT table_name||'.'||column_name||':'||data_type FROM information_schema.columns WHERE table_schema='public' AND (table_name,column_name) IN (('tickets','description'),('knowledge_documents','content'),('messages','body')) ORDER BY table_name"
    types=command(*FULL,'exec','-T','db','psql','-U','support','-d','support_os','-At','-c',sql).splitlines()
    assert len(types)==3 and all(row.endswith(':text') for row in types),types
    source_files=list((ROOT/'backend/rust-api/src').rglob('*.rs'))+list((ROOT/'backend/rust-api/migrations').glob('*.sql'))
    for path in source_files:
        source=path.read_text(encoding='utf-8')
        assert not re.search(r'\b(?:xmlparse|xmltable|xpath)\s*\(|::\s*xml\b|\bas\s+xml\b|std::process::Command|Command::new\(',source,re.I),str(path)
    forbidden={'xml','lxml','libxml2','defusedxml','bs4','subprocess','psycopg','asyncpg','sqlalchemy'}
    for path in (ROOT/'backend/ai-copilot/app').rglob('*.py'):
        tree=ast.parse(path.read_text(encoding='utf-8'))
        for node in ast.walk(tree):
            if isinstance(node,ast.Import):assert not any(n.name.split('.')[0] in forbidden for n in node.names),str(path)
            if isinstance(node,ast.ImportFrom):assert (node.module or '').split('.')[0] not in forbidden,str(path)
            if isinstance(node,ast.Call) and isinstance(node.func,ast.Name):assert node.func.id not in {'eval','exec','__import__'},str(path)
    report={'result':'passed','scope':'Current synthetic-data JSON API and hardened local rehearsal; not arbitrary direct SQL or future XML/upload features',
        'driver_host':platform.system(),'project':PREFIX,'runtime_boundaries':records,'source_digest':digest_sources(),
        'xml_protocol_rejected':True,'xml_text_stored_without_entity_expansion':True,
        'storage_types':types,'arbitrary_shell_tool_rejected':True,'health_probe_positive_and_negative':'passed',
        'source_boundary_review':'No XML SQL parser calls or Rust shell-command execution in current source/migrations'}
    (ROOT/'docs/STAGE1_RUNTIME_SECURITY.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))

if __name__=='__main__':main()
