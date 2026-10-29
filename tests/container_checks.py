"""Destructive-to-process checks restricted to our isolated Compose rehearsal.

Never removes volumes, resets a database, or operates on other projects.
Requires an already-built and started `aster-stage1-verify` Compose stack.
"""
import json
import pathlib
import subprocess
import time
import uuid
import httpx

ROOT = pathlib.Path(__file__).resolve().parents[1]
COMPOSE = ['docker', 'compose', '-p', 'aster-stage1-verify']
BASE = 'http://localhost:8080'
ORIGIN = 'http://localhost:3000'
report = {'environment': 'Docker Desktop Linux engine, release Rust, Python 3.12, Postgres 17, production standalone Next'}

def docker(*args):
    return subprocess.check_output(COMPOSE + list(args), cwd=ROOT, text=True).strip()

def wait_ready():
    started = time.monotonic()
    for _ in range(120):
        try:
            if httpx.get(BASE + '/ready', timeout=3).status_code == 200:
                return round((time.monotonic()-started)*1000)
        except httpx.HTTPError:
            pass
        time.sleep(1)
    raise AssertionError('Container API did not recover within 120 seconds')

def main():
    wait_ready()
    client = httpx.Client(base_url=BASE, headers={'Origin': ORIGIN}, timeout=30)
    created = client.post('/auth/signup', json={'email': f'container-{uuid.uuid4()}@example.test',
        'password': 'Synthetic-container-test-password!', 'organization_name': 'Synthetic restart fixture',
        'full_name': 'Synthetic container tester'})
    created.raise_for_status()
    headers = {'Authorization': 'Bearer '+created.json()['token']}
    client.cookies.clear()
    ticket = client.post('/tickets', headers=headers, json={'subject': 'Persist across container restart', 'description': 'Synthetic only'})
    ticket.raise_for_status()
    path = '/tickets/'+ticket.json()['id']
    migrations = docker('exec', '-T', 'db', 'psql', '-U', 'support', '-d', 'support_os', '-At', '-c',
        'SELECT version FROM _sqlx_migrations WHERE success ORDER BY version').splitlines()
    expected = sorted(p.name.split('_')[0] for p in (ROOT/'backend/rust-api/migrations').glob('*.sql'))
    assert sorted(migrations) == expected, (migrations, expected)
    report['migration_versions'] = migrations
    for service in ['api', 'ai', 'web']:
        assert docker('exec', '-T', service, 'id', '-u') != '0'
    bindings = json.loads(subprocess.check_output(['docker', 'inspect', '--format', '{{json .NetworkSettings.Ports}}', docker('ps', '-q', 'ai')], text=True))
    assert not any(bindings.values())
    assert docker('exec', '-T', 'ai', 'python', '-c', "import urllib.request,urllib.error; r=urllib.request.Request('http://127.0.0.1:8001/copilot/summarize',data=b'{}',headers={'Content-Type':'application/json'});\ntry: urllib.request.urlopen(r); raise SystemExit(1)\nexcept urllib.error.HTTPError as e: assert e.code==401") == ''
    report['non_root_and_private_ai'] = 'passed'
    denied = client.post('/tickets', headers=headers | {'Origin': 'https://attacker.example'}, json={'subject': 'CSRF'})
    assert denied.status_code == 403
    assert client.get('/inbox', headers=headers, params={'q': "' OR 1=1; --"}).json() == []
    report['csrf_bearer_origin_and_search_input'] = 'passed'
    # AI outage must not prevent manual customer support.
    docker('stop', 'ai')
    try:
        assert client.get(path+'/summary', headers=headers).status_code == 200
        manual = client.post(path+'/messages', headers=headers, json={'body': 'Manual support while AI is stopped', 'idempotency_key': str(uuid.uuid4())})
        manual.raise_for_status()
        assert any(a['status']=='fallback' for a in client.get('/ai/activity', headers=headers).json())
    finally:
        docker('start', '--wait', 'ai')
    report['ai_outage_manual_support'] = 'passed'
    recovery_start = time.monotonic()
    docker('restart', 'api', 'ai', 'web')
    wait_ready()
    report['service_restart_ready_ms'] = round((time.monotonic()-recovery_start)*1000)
    assert client.get(path, headers=headers).json()['subject'] == ticket.json()['subject']
    # A stopped DB must make readiness fail rather than advertise availability.
    docker('stop', 'db')
    try:
        assert httpx.get(BASE+'/ready', timeout=25).status_code == 503
    finally:
        recovery_start = time.monotonic()
        docker('start', '--wait', 'db')
    wait_ready()
    report['database_recovery_ms'] = round((time.monotonic()-recovery_start)*1000)
    assert client.get(path, headers=headers).status_code == 200
    report['database_readiness_failure_and_recovery'] = 'passed'
    docker('stop')
    recovery_start = time.monotonic()
    docker('start', '--wait')
    wait_ready()
    assert client.get(path, headers=headers).status_code == 200
    report['complete_stack_stop_start_preserves_data'] = 'passed'
    report['complete_stack_start_ready_ms'] = round((time.monotonic()-recovery_start)*1000)
    recovery_start = time.monotonic()
    docker('up', '-d', '--force-recreate', '--wait', '--wait-timeout', '180')
    wait_ready()
    assert client.get(path, headers=headers).status_code == 200
    report['container_recreation_preserves_data'] = 'passed'
    report['container_recreation_ready_ms'] = round((time.monotonic()-recovery_start)*1000)
    statuses = [client.post('/auth/login', json={}).status_code for _ in range(31)]
    assert statuses[-1] == 429 and all(s != 429 for s in statuses[:30]), statuses
    report['authentication_rate_limit'] = 'passed: 31st request denied'
    report['result'] = 'passed'
    (ROOT/'docs/STAGE1_CONTAINER_CHECKS.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(report, indent=2))

if __name__ == '__main__':
    main()
