"""Give the code and runtime assets one shared, content-derived cache revision."""
import hashlib
from pathlib import Path
import sys

directory = Path(sys.argv[1])
template = (directory / 'index.html').read_text()
assert '__LASTLIGHT_BUILD__' in template, 'Stamp the freshly copied HTML template'
digest = hashlib.sha256(template.encode())
files = [directory / 'lastlight.js', directory / 'lastlight_bg.wasm']
files += sorted(path for path in (directory / 'assets').rglob('*') if path.is_file())
for path in files:
    digest.update(path.relative_to(directory).as_posix().encode() + b'\0')
    digest.update(hashlib.sha256(path.read_bytes()).digest())
revision = digest.hexdigest()[:16]
(directory / 'index.html').write_text(template.replace('__LASTLIGHT_BUILD__', revision))
print(f'Browser build: {revision}')
