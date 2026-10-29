"""Rehearse both Docker layouts locally without deleting existing data.

Requires Docker's Linux engine, installed Python requirements, tests/npm ci and
Playwright Chromium. Fixed project names isolate these synthetic fixtures.
No deployment, paid integration, prune or volume deletion is performed.
"""
import os
from pathlib import Path
import secrets
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
os.chdir(ROOT)
def run(*command, cwd=ROOT, env=None):
    subprocess.run(list(command),cwd=cwd,env=env,check=True)

def main():
    target=ROOT/'.env'
    if not target.exists():
        target.write_text('\n'.join(f'{key}={secrets.token_hex(32)}' for key in
            ['POSTGRES_PASSWORD','JWT_SECRET','AI_SERVICE_TOKEN'])+'\n',encoding='utf-8')
    prefix=os.getenv('ASTER_VERIFY_PROJECT_PREFIX','aster-stage1')
    if prefix not in {'aster-stage1','aster-stage1-security'}:
        raise SystemExit('Only the two isolated rehearsal project prefixes are supported')
    api_port=int(os.getenv('API_PORT','8080'));web_port=int(os.getenv('WEB_PORT','3000'));beta_port=int(os.getenv('STAGE1_PORT','18083'))
    os.environ['FRONTEND_ORIGIN']=f'http://localhost:{web_port}'
    full=['docker','compose','-p',prefix+'-verify']
    beta=['docker','compose','-f','compose.stage1.yaml','-p',prefix+'-free-verify']
    run(*full,'build')
    run(*full,'up','-d','--wait','--wait-timeout','180')
    run('docker','run','--rm','--network','none','--mount',f'type=bind,source={ROOT},target=/verification,readonly',
        '-e','PYTHONPATH=/app','aster-stage1-verify-ai','python','-m','unittest','discover','-s','/verification/tests','-p','test_*.py')
    env={**os.environ,'TEST_API_URL':f'http://localhost:{api_port}','TEST_WEB_URL':f'http://localhost:{web_port}',
         'TEST_FRONTEND_ORIGIN':f'http://localhost:{web_port}','TEST_BETA_API_URL':f'http://localhost:{beta_port}'}
    for script in ['tests/integration.py','tests/stage1_integration.py']:
        run(sys.executable,script,env=env)
    run('node','node_modules/@playwright/test/cli.js','test',cwd=ROOT/'tests',env=env)
    run(sys.executable,'tests/container_checks.py',env=env)
    run(*beta,'build')
    run(*beta,'up','-d','--wait','--wait-timeout','180')
    run(sys.executable,'tests/stage1_integration.py',env=env|{'TEST_API_URL':f'http://localhost:{beta_port}'})
    run(sys.executable,'tests/free_container_checks.py',env=env)
    run(sys.executable,'tests/security_review.py',env=env)
    print('Container functional verification passed. Images, synthetic data and volumes retained.')

if __name__=='__main__':main()
