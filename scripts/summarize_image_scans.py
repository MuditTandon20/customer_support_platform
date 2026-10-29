"""Preserve Trivy HIGH/CRITICAL findings without suppressing unresolved risks."""
import argparse
import collections
import json
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument('--log-dir',type=Path,required=True)
args=parser.parse_args()
report={'scanner':'Trivy 0.76.1','scope':'HIGH/CRITICAL, OS and detected language packages; not an exhaustive security assessment','images':{}}
files={'api':'api-trixie-trivy.log','ai':'ai-final-trivy.log','web':'web-final-trivy.log',
       'combined_beta':'combined-minimal-trivy.log','local_postgres':'db-final-trivy.log'}
for role,name in files.items():
    raw=(args.log_dir/name).read_bytes()
    text=raw.decode('utf-16') if raw.startswith(b'\xff\xfe') else raw.decode('utf-8')
    for offset,c in enumerate(text):
        if c!='{':continue
        try:scan,_=json.JSONDecoder().raw_decode(text[offset:])
        except ValueError:continue
        if isinstance(scan,dict) and 'SchemaVersion' in scan:break
    else:raise SystemExit('No completed Trivy JSON in '+str(name))
    findings=[{'target':r['Target'],'id':v['VulnerabilityID'],'package':v['PkgName'],
               'installed':v['InstalledVersion'],'fixed':v.get('FixedVersion'),
               'severity':v['Severity'],'status':v.get('Status'),'url':v.get('PrimaryURL')}
              for r in scan.get('Results',[]) for v in r.get('Vulnerabilities',[])]
    report['images'][role]={'image':scan.get('ArtifactName'),'image_id':scan.get('Metadata',{}).get('ImageID'),
        'counts':dict(collections.Counter(f['severity'] for f in findings)),
        'fixable_findings':sum(bool(f['fixed']) for f in findings),'findings':findings}
report['readiness']='SECURITY HOLD: unresolved base-image findings; no clean image claim'
target=Path(__file__).resolve().parents[1]/'docs/STAGE1_IMAGE_SECURITY.json'
target.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:{'counts':v['counts'],'fixable_findings':v['fixable_findings']} for k,v in report['images'].items()},indent=2))
