#!/bin/sh
# Offline asset preparation only. None of these dependencies ship in the game.
set -eu
cd "$(dirname "$0")/.."
uv venv --python 3.14 .tools/animation --allow-existing
uv pip install --python .tools/animation/bin/python \
  numpy==2.5.2 pillow==12.3.0 opencv-python-headless==4.14.0.94 torch==2.14.0
mkdir -p .tools/rife
if [ ! -d .tools/rife/upstream/.git ]; then
  git clone https://github.com/hzwer/Practical-RIFE.git .tools/rife/upstream
fi
git -C .tools/rife/upstream checkout bbfd2ea90910789a860ea3e2b32a240cd577b75e
curl -L --fail 'https://drive.google.com/uc?export=download&id=1gViYvvQrtETBgU1w8axZSsr7YUuw31uy' -o .tools/rife/model.zip
printf '%s\n' 'c2452dd2b244947d4be580156bbead60d6b72af5736860f7d6b3f99648c9c4cc  .tools/rife/model.zip' | shasum -a 256 -c -
unzip -oq .tools/rife/model.zip 'train_log/*.py' 'train_log/flownet.pkl' -d .tools/rife/upstream
