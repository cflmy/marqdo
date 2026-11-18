#!/usr/bin/env python3
"""Build the executable proved policy and bind its source identity to the ABI."""
from pathlib import Path
import argparse,hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--lake',default=shutil.which('lake'));a=p.parse_args()
if not a.lake:raise SystemExit('lake is required to build the verified replay policy')
files=['ReplayPolicy/Cursor.lean','Main.lean','ReplayPolicy.lean','lakefile.lean','lean-toolchain','build_policy.py']
hashes={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in files}
digest=hashlib.sha256(json.dumps(hashes,sort_keys=True,separators=(',',':')).encode()).hexdigest()
(ROOT/'POLICY-SHA256').write_text(digest+'\n')
(ROOT/'ReplayPolicy/Identity.lean').write_text('namespace PolicyIdentity\ndef digest : String := "'+digest+'"\nend PolicyIdentity\n')
result=subprocess.run([a.lake,'build','replayPolicy'],cwd=ROOT)
if result.returncode:raise SystemExit(result.returncode)
changed=[name for name,digest_before in hashes.items()
         if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest_before]
if changed:raise SystemExit('policy sources changed during build: '+', '.join(changed))
binary=ROOT/'.lake/build/bin/replayPolicy'
(ROOT/'BUILD-IDENTITY.json').write_text(json.dumps(dict(policy_sha256=digest,source_sha256=hashes,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest()),indent=2)+'\n')
print(binary)
