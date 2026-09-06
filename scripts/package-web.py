#!/usr/bin/env python3
"""Package only the playable release files, never development starts."""
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

root = Path(__file__).resolve().parent.parent
dist = root / 'dist'
files = [dist / 'index.html', dist / 'lastlight.js', dist / 'lastlight_bg.wasm']
files.append(dist / '.nojekyll')
files += sorted((dist / 'assets').rglob('*'))
if any(not path.is_file() for path in files[:3]):
    raise SystemExit('Run ./scripts/build-web.sh first.')
output = root / 'releases' / 'lastlight-web.zip'
output.parent.mkdir(exist_ok=True)
with ZipFile(output, 'w', compression=ZIP_DEFLATED, compresslevel=9) as bundle:
    for path in files:
        if path.is_file():
            bundle.write(path, path.relative_to(dist))
print(output)
