"""Run the isolated upstream panic scenario without adding old crates to Aster."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root=Path(__file__).resolve().parents[1]
manifest=root/'tests/security-fixtures/block-buffer-010/Cargo.toml'
result=subprocess.run(['cargo','test','--manifest-path',str(manifest),'--locked'],capture_output=True,text=True,check=True)
print(result.stdout);print(result.stderr,file=sys.stderr)
assert 'advisory_inline_position_corruption_does_not_reproduce_in_010 ... ok' in result.stdout
assert '1 passed; 0 failed' in result.stdout
report={'result':'passed','crate':'block-buffer 0.10.4','case':'advisory_inline_position_corruption_does_not_reproduce_in_010',
    'fixture_lock_sha256':hashlib.sha256(manifest.with_name('Cargo.lock').read_bytes()).hexdigest(),
    'toolchain':subprocess.check_output(['cargo','--version'],text=True).strip(),
    'upstream':'https://github.com/RustCrypto/utils/security/advisories/GHSA-qwgh-2vcv-g2f7',
    'limitation':'Normal caught-panic regression plus source/version review, not a Miri run or general memory-safety proof. Fixture dependencies are not runtime dependencies.'}
(root/'docs/STAGE1_BLOCK_BUFFER_REPRODUCTION.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2))
