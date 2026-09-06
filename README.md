# Last Light

[Play in your browser](https://jacobious52.github.io/lastlight/)

A complete small exploration-horror game for desktop browsers, built in Rust with Bevy 0.18. The world is procedurally generated; painted character and creature animation atlases share its real-time lighting. The soundtrack uses two instrumental Suno v5.5 Pro compositions, with recorded CC0 footsteps, creature voices and mechanism effects. Asset sources and generation prompts are preserved in `art/`.

You move through a continuous, seeded cavern. A pulse reveals the way and wakes membranes, but organisms also hear it. Extinguishing makes you harder to follow. It also gives unobserved things a chance to move.

## AI-generated art disclaimer

The character, creature and decorative sprite artwork is **AI-generated using OpenAI ImageGen**, then processed and animated in code. It is not hand-drawn artwork. The instrumental music was generated with Suno Pro. Ground textures and sound effects use separately credited CC0 assets; the CC0 designation does not apply to the generated artwork or music. See the [asset credits](assets/CREDITS.md), [source notes](art/README.md), and [image-generation prompts](art/prompts.json) for provenance.

## Play locally

The browser build is the primary release. Requires a recent desktop browser with WebGL 2, keyboard and mouse. Headphones are recommended.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
./scripts/build-web.sh
./scripts/serve.sh
```

Open <http://localhost:8080>. Use `./scripts/build-web.sh dev` for a faster incremental development build. WASM must be served over HTTP; opening `index.html` as a file does not work.

Native macOS development:

```sh
cargo run --release
```

A fixed world can be selected using `?seed=12345` in the browser or `cargo run --release -- --seed=12345`. The default first world is seed 72419. A seed identifies a persistent geography; death does not regenerate it.

## Controls

| Input | Action |
| --- | --- |
| WASD / arrow keys | Move |
| Left click / hold | Move toward a point / steer continuously; slides along rock |
| Space / F | Release a revealing pulse |
| Hold Shift | Extinguish while held |
| X / right click | Toggle darkness |
| E | Set down a lantern after finding the ability; stay still for 1.15 seconds |
| R | Inspect / take / retrieve; hold to wake a structure |
| Q | Throw the found bell toward your facing direction |
| Tab | Recall illuminated places |
| Escape | Pause / resume |
| Enter | Begin / continue / return after death |
| M | Mute / unmute |
| N, then N in a menu | Generate a new world |

No health, ammunition, or upgrade-selection HUD. The body's light shows pulse readiness, proximity to danger unsettles it, and charging structures draw a ring around it. Mechanics are introduced by temporary contextual hints and brief ability-acquisition pauses. First contact flashes the traveller and screen edge; a failing lantern pulses and rattles.

## Journey

The guarded opening teaches movement, pulses, light-reactive membranes and extinction before entering the Hollow. Beyond it lie branching journeys to two abilities, three structures that must be awakened, optional small lights, a large organism, and a return to a final chamber. Both abilities offer additional routes through earlier spaces. Oil flasks extend a placed lantern from 28 to 60 seconds; only one burns at a time. Creatures snuff it after sustained contact. A reusable bell lures creatures without light and can be retrieved. Route stones mark the three main structures separately. Hold R at a structure while meeting its light condition. The memory view contains visited surfaces, surveyed destinations and barrier glyphs; it does not track creatures. Rootwork grips dark feet, stone galleries carry footsteps, and the deep silt holds a shadow that consumes placed light. The Hollow and final passage show three sockets corresponding to the awakened structures.

Sanctuaries, ability discoveries and completed resonators establish checkpoints. First contact gives a brief recovery window; repeated contact before the lantern steadies remains dangerous. Discoveries, opened shortcuts and geography survive death. Browser progress saves to local storage; native progress saves to `.lastlight-save.json` in the working directory. Saves are versioned and malformed saves are ignored. Older geography migrates checkpoints and shortcuts by landmark identity while retaining progress. Confirming a new world replaces the current run and saves its seed immediately.

## Build and verification

```sh
cargo test --locked
cargo fmt --check
cargo clippy --locked -- -D warnings
./scripts/build-web.sh
```

The generator checks progression and physical clearances, with deterministic tests across thousands of seeds. Gameplay tests exercise real ECS systems, collision, creature rules, light gates, both abilities, persistence, and the complete progression route. Automated coverage complements browser playtesting; it cannot establish whether the atmosphere or pacing works for every player.

The release output is the self-contained `dist/` directory. Run `python3 scripts/package-web.py` to create a portable `releases/lastlight-web.zip`. It can be hosted by any static web host serving `.wasm` as `application/wasm`. Serve WASM with gzip or Brotli for practical download times. No backend, account, API key, analytics, or network call beyond loading game files is needed. No COOP/COEP headers are required by the WebGL build.

## CI and GitHub Pages

[CI and deployment runs](https://github.com/Jacobious52/lastlight/actions/workflows/ci.yml)

Every push to `main` and pull request runs formatting, Clippy, the native gameplay tests, and a production WASM build. Rust is pinned in `rust-toolchain.toml`, Cargo dependencies are locked, and the WASM bindings tool matches the version used by the game. Build dependencies are cached between runs.

After both jobs pass on `main`, GitHub Actions publishes `dist/` to [GitHub Pages](https://jacobious52.github.io/lastlight/). Pull requests never deploy. The workflow can also be run manually from Actions. Developer playtest starts are excluded from production builds. All browser asset paths are relative, so the game works under the repository's `/lastlight/` path. Progress on the hosted game is stored in that browser and is separate from a localhost save.

## Project layout

- `src/world.rs`: deterministic spatial grammar, distance field, topology validation.
- `src/model.rs`: shared game state and creature components.
- `src/gameplay.rs`: movement, light, mechanisms, ecosystem, progression.
- `src/exploration.rs`: persistent finds, oil, route stones and a retrievable sound lure.
- `src/roots.rs`: branching roots shared between rendering and movement.
- `src/render.rs`, `assets/shaders/world.wgsl`: painted animation atlases, terrain and a sampled cave distance field in one lighting pass.
- `src/audio.rs`: recorded foley, creature voices and spatial/music scheduling.
- `src/save.rs`: versioned native/browser persistence.
- `src/ui.rs`: sparse game menus and contextual controls.
- `src/validation.rs`: adversarial gameplay and progression tests.
- `web/index.html`: browser loading shell and gesture-based audio unlocking.

See [world design](docs/WORLD.md), [rendering](docs/RENDERING.md), [audio](docs/AUDIO.md), and [playtest notes](docs/PLAYTEST.md) for implementation details and verification limits.

## Platform scope

Designed for desktop browsers and macOS. Touch controls and controller mapping are not implemented. Storage depends on the browser allowing local storage. Procedural geometry uses a constrained grammar: seeds change proportions, surface structure, and some routes while preserving authored progression roles. This is intentional rather than unrestricted random room placement.
