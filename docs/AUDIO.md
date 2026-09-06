# Audio

The runtime score contains two instrumental Suno v5.5 Pro compositions, downloaded through the user's account on 6 September 2026. Exploration lasts 144 seconds; the refuge variation lasts 100.48 seconds. Source pages, prompts and lossless masters are documented in `art/README.md`. Both platforms load local Vorbis files; no service is contacted during play.

All sound effects now use free CC0 recordings. Fantozzi supplies stone and sand footsteps, Kenney supplies scuffs and material impacts, and pauliuw and qubodup supply creature voices. Full provenance and licenses ship in `assets/CREDITS.md`. There are no runtime oscillators, generated effects, pressure beds or white-noise fallback loops.

`scripts/prepare-foley.py` prepares twelve footsteps and the stopping scuff. `scripts/prepare-effects.py` trims, retunes, filters, gain-matches and adds restrained reflections to recorded creature/mechanism sounds. Source archives are retained in `art/audio-source/`. These processing scripts are reproducible; they do not synthesize replacement voices.

## Movement and signals

Walking advances a 76-unit animation cycle with contacts every 38 units. The audio cue and opaque animation use the same contact phase. Stopping settles into the next planted pose without adding walking sounds. Steps are softer in darkness; the lower world selects sand takes. Stone-gallery footfalls also inform Listener sensing, with a faint floor echo; dark steps are too quiet to create that extra signal. Scuffs accompany stopping or striking geometry, with a small body impulse.

| Source | Recorded signature |
| --- | --- |
| Pulse and membranes | Soft plate and wood resonances with short reflections |
| Anchor and discoveries | Muted bell, glass and plate contacts |
| Listener | A breath/growl; an immediate stronger warning when it detects a signal |
| Still | A slowed, strained voice while moving; quiet when held under observation |
| Grazer | A separate quieter, lower creature call |
| Leviathan | A long, low voiced groan, spaced far enough apart to avoid stacking |
| First contact | Softened recorded heavy impact accompanying the body flash and stumble |
| Near death | Repeated, low recorded metal rattle while the lantern is unstable |
| Death | Material impact with a subdued tail |
| Ending | A finite recorded resonance; the existing score fades out |

Creature voices follow their physical positions. Ears are 180 world units apart, with spatial scale 1/350. Distance attenuates them; rock and closed membranes select a muffled version of the same signature. This is stereo positioning and simple occlusion, not binaural HRTF or full acoustic simulation. At most seven positioned voices can coexist. Each organism has a cooldown; detecting the player can produce an immediate warning instead of waiting for its ambient interval.

## Music and browser startup

The browser wrapper tracks AudioContexts before WASM starts and resumes them inside keyboard or pointer gestures. The title is silent. About one second after Begin/Continue, the first ready music asset starts, even if danger is present. Scheduling follows actual track completion and leaves at least 24 seconds of quiet between pieces. Sanctuary playback can select the refuge variation. Music ducks during danger and fades out at the conclusion.

Pause freezes existing voices and the scheduler. Mute fades existing non-spatial voices, silences positioned voices, and discards new cues instead of accumulating them. The same audio assets are used in browser and native builds.

`#game-status` exposes read-only music state, filename, audio-sink position, gain and footstep count. The continuous campaign playthrough verified advancing music through the ending. These diagnostics prove playback, not subjective listening quality. Human headphone and laptop-speaker listening remains valuable for tonal balance.

## Integration and checks

Gameplay queues `Cue` values into `AudioCues`; `soundscape` runs after gameplay. Creature warnings schedule from their own detection state rather than a generic click cue. Mechanism wake cues have a separate cooldown. Finished voices despawn.

Gameplay tests verify ground-distance footfalls, silent idle/blocked movement and planted stopping. Browser checks cover gesture unlock, advancing music, pause/resume and runtime errors. Prepared files have bounded peaks and fades. Recheck stereo direction and clear/muffled transitions when editing creature voices; gain measurements alone cannot establish that a warning is readable.

The retrievable bell uses Kenney’s CC0 `impactBell_heavy_000.ogg`, slightly slowed and reflected by `scripts/prepare-effects.py`. Its source is spatialized at the landing point. It has a dedicated cue, separate from creature warning sounds.
