# Sound recordings

**Fantozzi — Footsteps (Grass/Sand & Stone)**, edited into individual steps by
qubodup. CC0 1.0 public-domain dedication.
https://opengameart.org/content/fantozzis-footsteps-grasssand-stone
Original pack: https://freesound.org/people/Fantozzi/packs/10338/

The twelve stone/sand recordings supply `audio/foley/stone-*.ogg` and
`audio/foley/sand-*.ogg`. Filtered, trimmed, softened and gain matched by
`scripts/prepare-foley.py`. Original lossless files and downloaded archive remain
in `art/audio-source/foley/`.

**Kenney — Impact Sounds**, CC0 1.0 public-domain dedication.
https://kenney.nl/assets/impact-sounds
The carpet footstep `footstep_carpet_002.ogg` supplies the quiet stopping scuff,
filtered and attenuated as `audio/foley/scuff.ogg`.

License: https://creativecommons.org/publicdomain/zero/1.0/

The music uses the project's two Suno Pro v5.5 instrumentals; source WAVs and
generation prompts are documented in `art/README.md`. All effects use the CC0 recordings listed here. No runtime synthesized effects remain.

**pauliuw — Silent beast growls (4)**, CC0 1.0.
https://opengameart.org/content/silent-beast-growls4
The first and third recordings provide Listener inhalations/warnings and Grazer calls.

**qubodup — Ghost Monster Voice Moaning & Growling**, CC0 1.0.
https://opengameart.org/content/ghost-monster-voice-moaning-growling
Recordings 02 and 05 provide the Weight and Still voices.

Kenney's recorded plate, bell, glass, wood and soft impacts provide all
mechanism, discovery, arrival and death cues. `scripts/prepare-effects.py`
reproducibly trims, filters, retunes and adds reflections to these recordings.
The original archives remain in `art/audio-source`; effects are distributed in
`audio/effects`. Clear and occluded versions retain the same creature signature.

Kenney's `impactPunch_heavy_002.ogg` supplies the first-contact impact and
`impactMetal_heavy_002.ogg` the critical lantern rattle. Both are retuned,
filtered, gain matched and faded by `scripts/prepare-effects.py`; the same CC0
license applies.

# Ground textures

Poly Haven assets are CC0 1.0: https://polyhaven.com/license

- Gravel Floor — Jenelle van Heerden / Matterfield: https://polyhaven.com/a/gravel_floor
- Forrest Ground 01 — Rob Tuytel: https://polyhaven.com/a/forrest_ground_01
- Embedded Rock Floor — Dimitrios Savva: https://polyhaven.com/a/embedded_rock_floor
- Brown Mud 02 — Rob Tuytel: https://polyhaven.com/a/brown_mud_02

Original 1K diffuse JPEGs and exact download URLs remain in `art/texture-source/`. `scripts/prepare-surfaces.py` converts them to restrained grayscale and packs the four material regions. Only these downloaded asset files are redistributed, not website preview renders.

The thrown bell uses Kenney's `impactBell_heavy_000.ogg`, under the same CC0 license as the other impacts.
