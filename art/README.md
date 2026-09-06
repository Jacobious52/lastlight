# Source artwork and score

Visual generation used the built-in `image_gen.imagegen` tool. Exact prompts and accepted source/runtime paths are in [prompts.json](prompts.json). Original RGBA sheets remain in [source](source); packed textures are in [../assets/art](../assets/art). `scripts/pack-art.py` uses Python, Pillow and NumPy to extract alpha silhouettes, scale uniformly and align boot contacts.

Two instrumental soundtrack masters were generated through the user's Suno Pro account with v5.5 on 2026-09-06. Lyrics were empty and the style prompt excluded voices, choir and lyrics. They were unlocked through Suno's download interface. Lossless sources are [Stone and Breath](audio-source/stone-and-breath.wav) (144 seconds) and [Refuge variation](audio-source/stone-and-breath-refuge.wav) (100.48 seconds).

Source song pages:
- https://suno.com/song/3bf0d271-f8e5-494c-8f72-c5d3fcbd67f7
- https://suno.com/song/f2b51e2c-0b12-40c1-8200-e864d35898a6

Both used this style prompt:

> Instrumental only. No vocals, no choir, no lyrics. Haunting sparse electroacoustic soundscape for a slow monochrome exploration horror game. Deep bowed contrabass, strained cello harmonics, distant felt piano single notes, granular tape decay, almost inaudible glass resonance. Physical, intimate, unsettling. Very slow irregular phrases separated by long spaces, unresolved minor seconds, melancholy and dread with moments of fragile beauty. No beat, no drums, no arpeggios, no song structure, no epic build, no bright synth pads, no jump scare hits. Four minute evolving piece with quiet entrance and long decaying tail.

Runtime Ogg files are normalized to -24 LUFS with -3 dB true-peak headroom, quiet entrances and ten-second ending fades. `scripts/prepare-score.sh` reproduces them. No runtime service connection is required.

Movement audio now uses freely licensed CC0 recordings; see [credits](../assets/CREDITS.md). The source archives and lossless steps are preserved under `audio-source/foley/`, and `scripts/prepare-foley.py` reproduces the filtered runtime sounds.

Creature animation keyframes are expanded into 48-pose cycles by `scripts/interpolate-animation.py` (NumPy and OpenCV). It estimates bidirectional motion and splats pixels forward to preserve occluding limbs. Intermediate motion data stays in `motion-fields/`; playable atlases are `assets/art/*-motion.png`. The original image-generation prompts are unchanged.

The traveller uses `scripts/animate-traveller.py` and the new complete-body source poses in `source/traveller-v2/`. The exact built-in imagegen prompts accompany those sheets. `scripts/setup-animation.sh` installs the pinned offline tooling and downloads the checksum-verified Practical-RIFE v4.26 model. RIFE estimates intermediate full-body poses and occlusion; the script carries alpha through those same warps, removes matte contamination, and restores solid cloth coverage to avoid translucent boots. It bakes 48 walking poses per direction and 24 settling poses for each contact/direction. No model is downloaded or run by players. The old cut-up-leg IK script has been removed.

[Practical-RIFE](https://github.com/hzwer/Practical-RIFE) code and model weights are MIT-licensed by their authors. They are build tools only, downloaded into ignored `.tools/`; their upstream license remains there. The generated PNG atlases, rather than the model or inference code, ship in Last Light. The pinned revision and model checksum are recorded in `scripts/setup-animation.sh`.

Regenerate the traveller with:

```sh
./scripts/setup-animation.sh
.tools/animation/bin/python scripts/animate-traveller.py
```

The bake also measures the actual flame in all 288 poses and writes `src/traveller_lamps.rs`. Regenerate those coordinates with the atlases so illumination remains attached to the physical lantern.

The latest built-in image-generation calls used generation mode (no reference images): `source/objects.png` supplies eight distinct props including the lantern, and `source/pickups.png` supplies the bell and oil flask. Their exact prompt set is in `prompts.json`. `scripts/pack-art.py` packs both into `assets/art/objects.png` as four columns by three rows.

Ground material scans are downloaded CC0 assets from Poly Haven, not generated images. Original JPEGs and URL/size/license records are in `texture-source/`; `scripts/prepare-surfaces.py` builds `assets/art/surfaces.png`. See the shipped asset credits for authors and primary source links.
