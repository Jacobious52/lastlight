# Rendering Last Light

The Rust generator rasterizes signed floor distance into an RGBA texture at roughly eight world units per texel. Red stores clearance, green the region and blue the nearest progression landmark. Concave shelves, bent passages and solid cover use the same field for movement, sight and lighting.

The fullscreen WebGL 2 material combines this field with grayscale painted textures and alpha animation atlases. Generated source sheets supply diverse props, physical pickups, the traveller and three creature families. Four CC0 Poly Haven scans supply the regional ground materials. The deep shadow is a full-resolution procedural volume of seven curling, tapered branches. It overlays terrain rather than scaling a walking creature. Its 56 segments and contact radii are shared with the simulation. The traveller uses `scripts/animate-traveller.py`: regenerated complete body poses, with RIFE in-between frames baked offline. Hood, shoulders, arms, hips, backpack, coat and legs move together. Creature cycles retain offline motion warping from `scripts/interpolate-animation.py`. The shader samples one finished RGBA pose per silhouette; it never cross-fades two different bodies at runtime.

`art/prompts.json` records the exact built-in image-generation prompts. Originals live in `art/source/`; runtime atlases live in `assets/art/`. `scripts/pack-art.py` finds separate silhouettes from genuine alpha, extracts them without cutting through nominal grid boundaries, and aligns ground contacts in padded cells. The traveller regeneration prompts live in `art/source/traveller-v2/prompts.json`. These source sheets returned an opaque light checkerboard despite the transparency request; the animation script removes that matte before packing and preserves the enclosed lantern glass.

Gait advances with actual travel. The traveller cycle covers 76 world units, so normal and dark cruising speeds advance about 52 and 39 baked poses per second respectively, without alpha cross-fading. A stopped creature freezes its animation. The traveller finishes the current half-step into a planted pose; idle settling emits no walking sound. A separate atlas settles either contact into a generated standing pose over 24 frames, then continuous upper-body sway and breathing keep the resting traveller alive without moving its boots. Blocked movement does not walk in place. Horizontal facing persists across stops; up/down movement uses the corresponding traveller view. Boots, body compression and camera response share the distance-based footfall phase.

Props attach to real outcrops or solid boundary ledges; entrances carved through a boundary reject that decoration. Foreground overlap uses reduced opacity over the player. Artwork passes through local light; the world remains gray while the lantern has a restrained amber tint. Ground sampling uses isotropic world coordinates and four stochastically offset, quarter-turned photographic samples. Bilinear blending hides repetition without mirroring or stretching material features. Decorative shader stripes and the repeated slab grid have been removed. Prop families depend on region and room, with at most one masonry arch per chamber. Each barrier has a distinct silhouette: iris, thorns, paired ribs, woven cloth or three sockets. Its gap follows the real mechanism opening; the memory view repeats the corresponding glyph.

The ordinary personal light has a radius parameter of about 77 world units and a much lower gain; its useful reach is a small pool around the feet. A pulse still reveals the wider route. Anchor and sanctuary light provide distinct local relief. The flame is measured separately in every walking and settling frame. Rust selects the pose and its flame coordinate together, and the shader applies the same facing, breathing and placement transforms to both. Three slow, low-amplitude flicker components modulate its warm glow. Floor-visibility rays start at the ground contact, keeping a lamp painted above a wall from swallowing the traveller’s footing. Sampled shadow rays hide surfaces behind rock. Creatures do not remain in the memory view. The camera follows smoothly with a 390-world-unit height; width follows viewport aspect ratio. Uniform arrays and CPU culling cap shader work independently of total world size.

Keep the smooth final fade into deep rock. A deep-rock early fragment return previously exposed clipped Bevy menu text in browser testing. Shader edits need a reload and asset copy; uniform or texture binding changes also need a Rust rebuild.

Both browser build scripts stamp a content revision over the JavaScript, WASM,
HTML and runtime assets. The loader applies it to module/WASM URLs and same-origin
asset fetches, so cached artwork cannot be mixed with a new atlas layout. The
revision is available as `data-build` on the accessible game-status output.

Walking atlases use eight columns, six rows per cycle. The six settling sequences share a twelve-column atlas. All stay within 3072 pixels on either axis. Runtime rendering needs one artwork sample per animated silhouette; motion estimation runs offline, never in the browser.


First contact briefly lights the body and screen edges, jolts the camera, and
makes the traveller stumble. Rising danger produces an amber, pulsing constriction
and irregular lamp output. This gives feedback without a health bar. Map framing
reserves space below the geography for the objective and earned controls.


The lighting pass renders to a linear-filtered image capped at 1.25 samples per
logical pixel and 1,600 pixels on its longest edge. A separate camera presents
that image with native-resolution UI. This bounds fragment cost on Retina displays
and at increased browser zoom. The target is resized only when its dimensions
change; unchanged images must not receive mutable asset access every frame.

Rootwork generates tapered, branching curves from nearby wall attachments. The CPU retracts their actual endpoints toward the attachment under personal light, pulses or placed lanterns, and regrows them after a short dark interval. Movement queries the same segments. Visible roots are culled before upload; the material skips pixels outside each segment bounding box.

The placed lantern has an iron/glass sprite with a measured warm flame offset. Setting it down lowers it beside the traveller over 1.15 seconds and briefly bends the upper body. Its last four seconds dim, creature contact makes it flutter before extinction, and its dark shell remains until replaced. The stone footfall ring is shorter, thinner and roughly one fifth as bright, without weakening the audible or AI signal.
