"""Query public OSV advisories using dependency names/versions only; no repository data is uploaded."""
import importlib.metadata as metadata
import json
import pathlib
import re
import subprocess
import argparse
import httpx
try:
    from packaging.requirements import Requirement
except ImportError:
    from pip._vendor.packaging.requirements import Requirement

parser=argparse.ArgumentParser()
parser.add_argument('--target',help='Cargo target triple for active dependency inventory')
parser.add_argument('--python-image',help='Inspect every installed distribution in a local Docker image, without networking')
parser.add_argument('--output',default='docs/dependency-scan.json')
args=parser.parse_args()
root=pathlib.Path(__file__).parents[1]
packages=[]
lock=(root/'backend/rust-api/Cargo.lock').read_text(encoding='utf-8')
tree=subprocess.run(['cargo','tree','--locked','--manifest-path',str(root/'backend/rust-api/Cargo.toml'),'--prefix','none']+(['--target',args.target] if args.target else []),capture_output=True,text=True,check=True).stdout
active=set(re.findall(r'^([\w-]+) v([^\s]+)',tree,re.M))
for block in lock.split('[[package]]')[1:]:
    name=re.search(r'^name = "([^"]+)"',block,re.M)
    version=re.search(r'^version = "([^"]+)"',block,re.M)
    if name and version and 'registry+' in block:
        packages.append({'package':{'name':name[1],'ecosystem':'crates.io'},'version':version[1]})
if args.python_image:
    inventory=json.loads(subprocess.check_output(['docker','run','--rm','--network','none',args.python_image,'python','-c',"import importlib.metadata as m,json;print(json.dumps([{'name':d.metadata['Name'],'version':d.version} for d in m.distributions()]))"],text=True))
    packages.extend({'package':{'name':d['name'].lower().replace('_','-'),'ecosystem':'PyPI'},'version':d['version']} for d in inventory)
queue=[] if args.python_image else ['fastapi','uvicorn[standard]','pydantic','pydantic-settings','python-dotenv','openai','httpx']
seen=set()
while queue:
    req=Requirement(queue.pop());name=req.name.lower().replace('_','-')
    if name in seen:continue
    seen.add(name)
    try:distribution=metadata.distribution(name)
    except metadata.PackageNotFoundError:continue
    packages.append({'package':{'name':name,'ecosystem':'PyPI'},'version':distribution.version})
    for dependency in distribution.requires or []:
        item=Requirement(dependency)
        if item.marker is None or any(item.marker.evaluate({'extra':extra}) for extra in set(req.extras)|{''}):queue.append(str(item))
findings=[]
with httpx.Client(timeout=60) as client:
    for offset in range(0,len(packages),100):
        batch=packages[offset:offset+100]
        response=client.post('https://api.osv.dev/v1/querybatch',json={'queries':batch});response.raise_for_status()
        for query,result in zip(batch,response.json()['results']):
            if result.get('vulns'):
                findings.append(query|{'active_in_current_target':query['package']['ecosystem']=='PyPI' or (query['package']['name'],query['version']) in active,'advisories':[v['id'] for v in result['vulns']]})
report={'source':'https://api.osv.dev/v1/querybatch','dependencies_checked':len(packages),'cargo_target':args.target or 'native','python_scope':('All installed distributions in local image '+args.python_image) if args.python_image else 'Interpreter running this script; recursive project dependencies including uvicorn standard extras','findings':findings}
(root/args.output).write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2))
