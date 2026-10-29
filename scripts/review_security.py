"""Retain raw scans and assess only explicitly reviewed Stage 1 execution paths.

Unknown findings, affected Go symbols, missing runtime evidence or source drift
fail the release gate. This does not add ignore rules to either scanner.
"""
import argparse
import collections
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]
PREFIX=os.getenv('ASTER_VERIFY_PROJECT_PREFIX','aster-stage1')
assert PREFIX in {'aster-stage1','aster-stage1-security'}
EVIDENCE=ROOT/'.security-evidence'
TRIVY='aquasec/trivy:0.76.1'
SCANNER='aster-security-govulncheck:1.9.0'

def run(*args,output=None):
    if output:
        with output.open('wb') as stream:subprocess.run(args,cwd=ROOT,stdout=stream,check=True)
    else:subprocess.run(args,cwd=ROOT,check=True)
def capture(*args):return subprocess.check_output(args,cwd=ROOT,text=True).strip()
def digest_sources():
    files=[]
    for directory,patterns in [('backend/rust-api/src',['*.rs']),('backend/rust-api/migrations',['*.sql']),
                               ('backend/ai-copilot/app',['*.py']),('frontend/src',['*.ts','*.tsx'])]:
        for pattern in patterns:files.extend((ROOT/directory).rglob(pattern))
    files.extend([ROOT/'backend/rust-api/Dockerfile',ROOT/'backend/ai-copilot/Dockerfile',ROOT/'frontend/Dockerfile'])
    files.extend((ROOT/'deploy').glob('Dockerfile.*'))
    files.extend(ROOT.glob('compose*.yaml'))
    files.extend([ROOT/'render.yaml',ROOT/'vercel.json',ROOT/'frontend/vercel.json'])
    files.extend([ROOT/'backend/rust-api/Cargo.toml',ROOT/'backend/rust-api/Cargo.lock',
                  ROOT/'backend/ai-copilot/requirements.txt',ROOT/'backend/ai-copilot/requirements.lock',
                  ROOT/'frontend/package.json',ROOT/'frontend/package-lock.json'])
    # Git checkouts may use CRLF; line-ending differences are not source changes.
    entries=[str(p.relative_to(ROOT)).replace('\\','/')+':'+hashlib.sha256(p.read_bytes().replace(b'\r\n',b'\n')).hexdigest()+'\n' for p in files]
    return hashlib.sha256(''.join(sorted(entries)).encode()).hexdigest()
def parse_events(path):
    text=path.read_text(encoding='utf-8-sig');decoder=json.JSONDecoder();events=[]
    while text.strip():
        text=text.lstrip();event,n=decoder.raw_decode(text);events.append(event);text=text[n:]
    return events

RULES={
 'CVE-2026-76642':('mount_hooks','Privileged mount helpers/hooks are not invoked; no SUID/SGID executables, no effective capabilities and no-new-privileges are verified.'),
 'CVE-2026-78408':('namespace_operator','No privileged nsenter/cgroup-join operation or host PID namespace is exposed; effective capabilities are zero.'),
 'CVE-2026-78409':('mount_hooks','No fstab-authorized mounts, privileged mount calls or SUID executables are available to application requests.'),
 'CVE-2026-78410':('mount_hooks','The privileged SUID mount/fstab preconditions are absent and the container root filesystem is read-only.'),
 'CVE-2026-16742':('component_absent','systemd-homed is absent; application containers do not host homed-managed interactive users.'),
 'CVE-2026-54369':('unused_privileged_acl','No user-controlled filesystem-path/ACL operation or arbitrary process tool is exposed. Processes are non-root, capability-free and cannot gain privileges.'),
 'CVE-2025-69720':('unused_cli','The affected infocmp command is not invoked on user terminfo; there is no terminal/command execution feature.'),
 'CVE-2026-9538':('unused_archive_parser','The application has no Perl Archive::Tar ingestion or executable file-upload path. Current inputs are bounded JSON/text.'),
 'CVE-2026-24882':('component_absent','tpm2daemon and TPM devices are absent. The running database does not invoke GnuPG/TPM decryption.'),
 'CVE-2026-6653':('unused_xml_parser','Only the private PostgreSQL image contains the affected parser. Public inputs are JSON and TEXT, not XML SQL operations; raw XML is rejected and XML-looking text is stored without entity expansion.'),
 'CVE-2026-74860':('unused_xml_parser','PostgreSQL does not expose the libxml2 Python SAX binding to Aster; there is no Python/XML ingestion path.'),
 'CVE-2026-86138':('unused_xml_parser','No XML SQL type/parser or dictionary API is invoked from the current JSON/TEXT support routes.'),
 'CVE-2026-86139':('unused_xml_parser','No libxml2 URI-escaping operation is invoked by current application routes.'),
 'CVE-2026-86140':('unused_xml_parser','No XML DTD validation operation is exposed by the current application.'),
 'CVE-2026-86142':('unused_xml_parser','No XPath/XPointer evaluation is exposed by the current application.'),
 'CVE-2026-86143':('unused_xml_parser','No XML output callback is invoked from application routes.'),
 'CVE-2026-86144':('unused_xml_parser','No XInclude, XML resource loader or arbitrary URL/file ingestion is exposed.'),
}

def assess_finding(row,role,advisories,findings):
    if row['package']=='stdlib':
        # Never apply gosu's binary analysis to another Go binary or image.
        if role!='local_postgres' or row['target'].lstrip('/')!='usr/local/bin/gosu':return None
        advisory=advisories.get(row['id'])
        matched=[f for f in findings if advisory and f['osv']==advisory['id']]
        if any(t.get('function') for f in matched for t in f.get('trace',[])):return None
        imports=[i for a in (advisory or {}).get('affected',[]) for i in a.get('ecosystem_specific',{}).get('imports',[])]
        package_present=any(t.get('package') for f in matched for t in f.get('trace',[]))
        if imports and matched and (not package_present or any(i.get('symbols') for i in imports)):
            return ('affected_symbol_absent','Exact gosu binary has version-level metadata matches but govulncheck reports no affected linked symbols for this scoped Go advisory.','https://pkg.go.dev/vuln/'+advisory['id'])
        return None
    if row['id'] not in RULES:return None
    category,reason=RULES[row['id']]
    if category=='unused_xml_parser' and role!='local_postgres':return None
    return category,reason,'https://security-tracker.debian.org/tracker/'+row['id']

def main():
    EVIDENCE.mkdir(exist_ok=True)
    runtime=json.loads((ROOT/'docs/STAGE1_RUNTIME_SECURITY.json').read_text())
    assert runtime['result']=='passed' and runtime['project']==PREFIX
    assert runtime['source_digest']==digest_sources(),'Source changed after runtime boundary verification'
    scope=json.loads((ROOT/'docs/STAGE1_REVIEW_SCOPE.json').read_text())
    assert scope['reviewed_source_digest']==runtime['source_digest'],'Source/config/dependency changes require a fresh security review and scope update'
    images={r['service']:r['image_id'] for r in runtime['runtime_boundaries']}
    run('docker','build','-f','deploy/Dockerfile.govulncheck','-t',SCANNER,'.')
    dbid=capture('docker','compose','-p',PREFIX+'-verify','ps','-q','db')
    assert capture('docker','inspect','--format','{{.Image}}',dbid)==images['db']
    run('docker','cp',dbid+':/usr/local/bin/gosu',str(EVIDENCE/'gosu'))
    run('docker','run','--rm','--mount',f'type=bind,source={EVIDENCE},target=/evidence,readonly',SCANNER,
        '-json','-mode=binary','/evidence/gosu',output=EVIDENCE/'gosu.govulncheck.json')
    events=parse_events(EVIDENCE/'gosu.govulncheck.json')
    findings=[e['finding'] for e in events if 'finding' in e]
    linked=[f for f in findings if any(t.get('function') for t in f.get('trace',[]))]
    advisories={a:e['osv'] for e in events if 'osv' in e for a in [e['osv']['id']]+e['osv'].get('aliases',[])}
    go_report={'binary_sha256':hashlib.sha256((EVIDENCE/'gosu').read_bytes()).hexdigest(),
        'version':capture('docker','compose','-p',PREFIX+'-verify','exec','-T','db','gosu','--version'),
        'scanner':next(e['config'] for e in events if 'config' in e),'module_package_and_symbol_findings':findings,
        'affected_symbol_findings':linked,'limitation':'Binary symbol analysis, not a complete source-level call graph or general proof of absence of vulnerabilities.'}
    (ROOT/'docs/STAGE1_GOSU_ANALYSIS.json').write_text(json.dumps(go_report,indent=2)+'\n',encoding='utf-8')
    assert not linked,'Affected gosu symbols detected; review or update the utility before release'
    raw={'scanner':'Trivy 0.76.1','scope':'HIGH/CRITICAL, no ignore rules; duplicate package matches retained','images':{}}
    assessments=[];unknown=[]
    role_map={'api':'api','ai':'ai','web':'web','combined_beta':'backend','local_postgres':'db'}
    for role,service in role_map.items():
        image=images[service]
        path=EVIDENCE/(role+'.trivy.json')
        run('docker','run','--rm','-v','/var/run/docker.sock:/var/run/docker.sock',
            '-v','aster-stage1-security-trivy-cache:/root/.cache/',TRIVY,'image','--scanners','vuln',
            '--severity','HIGH,CRITICAL','--format','json',image,output=path)
        scan=json.loads(path.read_text());rows=[]
        for result in scan.get('Results',[]):
            for v in result.get('Vulnerabilities',[]):
                row={'target':result['Target'],'id':v['VulnerabilityID'],'package':v['PkgName'],
                     'installed':v['InstalledVersion'],'fixed':v.get('FixedVersion'),'severity':v['Severity'],
                     'status':v.get('Status'),'url':v.get('PrimaryURL')}
                rows.append(row)
                assessment=assess_finding(row,role,advisories,findings)
                if assessment is None:unknown.append(role+':'+row['id']);continue
                category,reason,source=assessment
                assessments.append({'product_image':image,'role':role,'id':row['id'],'component':row['package'],
                    'installed':row['installed'],'disposition':'not_affected_in_current_stage1_execution',
                    'category':category,'rationale':reason,
                    'source':source})
        raw['images'][role]={'image':image,'image_id':image,'counts':dict(collections.Counter(r['severity'] for r in rows)),
            'fixable_findings':sum(bool(r['fixed']) for r in rows),'findings':rows}
    raw['readiness']='Raw findings remain; see STAGE1_SECURITY_REVIEW.json for scoped assessment'
    (ROOT/'docs/STAGE1_IMAGE_SECURITY.json').write_text(json.dumps(raw,indent=2)+'\n',encoding='utf-8')
    reproduction=json.loads((ROOT/'docs/STAGE1_BLOCK_BUFFER_REPRODUCTION.json').read_text())
    assert reproduction['result']=='passed'
    assert reproduction['fixture_lock_sha256']==hashlib.sha256((ROOT/'tests/security-fixtures/block-buffer-010/Cargo.lock').read_bytes()).hexdigest()
    report={'scope':'Synthetic, non-commercial Stage 1 beta only; not arbitrary SQL access, shells, XML/uploads, privileged host administration or production certification',
        'source_digest':runtime['source_digest'],'images':images,'runtime_evidence':'STAGE1_RUNTIME_SECURITY.json',
        'govulncheck_evidence':'STAGE1_GOSU_ANALYSIS.json','assessments':assessments,'unreviewed_findings':unknown,
        'rust_advisory_review':{'id':'GHSA-qwgh-2vcv-g2f7','installed':'block-buffer 0.10.4',
            'disposition':'not_affected_by_the_reviewed_inline_cursor_bug',
            'basis':'Upstream affected range begins at 0.11.0; 0.10.4 stores its cursor separately and passes the isolated caught-panic reproduction.',
            'evidence':'STAGE1_BLOCK_BUFFER_REPRODUCTION.json','raw_osv_finding_retained':True,
            'source':'https://github.com/RustCrypto/utils/security/advisories/GHSA-qwgh-2vcv-g2f7'},
        'decision':'eligible_for_stage1_test_deployment' if not unknown else 'security_hold',
        'reassessment_required_on':['source/image/config changes','new scanner findings','XML/file/archive/URL ingestion','shell tools or untrusted direct SQL','relaxing non-root/no-SUID controls','real customer data or commercial production use']}
    (ROOT/'docs/STAGE1_SECURITY_REVIEW.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({'decision':report['decision'],'unreviewed_findings':unknown,'affected_gosu_symbols':len(linked)},indent=2))
    assert not unknown,'Unreviewed image findings remain; public security hold is retained'

if __name__=='__main__':main()
