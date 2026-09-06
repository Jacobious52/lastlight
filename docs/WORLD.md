# Last Light world grammar

The world is one continuous, persistent space. A seed changes chamber proportions,
positions, passage widths, edge shapes and optional cross-links; it does not discard
the authored teaching sequence or shuffle required abilities behind their own locks.
The approximate extent is 6,400 × 5,900 world units. Concave polygon shelves, fractured galleries and recesses merge into bent, variable-width passage spines. Room anchors identify progression landmarks, not circular cavities. Passage `width` means the half-width of the protected gate throat; wider shoulders and offset approaches vary independently. The field is positive inside navigable terrain; dynamic gates are separate.

The route begins at Cradle (0), moves through the light-reactive Fold gate into
Fold (1), then through a darkness-reactive membrane to Witness (2). Witness is
the controlled first encounter. Its initial creature should be slow and constrained
to its own room. Hollow (3) is the central sanctuary and navigation landmark.

From Hollow the northern circuit visits Listening Well (4), Split Nerve (5), and
Still Hearth (6), where Anchor is acquired. Rest (7) provides a haven. The gate
between Rest and Return (8) accepts Anchor and closes the first learned loop.
An Anchor seam south of Hollow opens Curtain (9) and Absent Garden (10), where
Veil is learned. Veil immediately opens the way to Afterimage (11); it also
opens the previously visible northern route from Split Nerve to Sealed Lung (16).

Sealed Lung's resonator retracts from direct personal light. Leave an Anchor
95–220 units from its centre, extinguish, and approach the centre without
pulsing, then hold R for 3.5 seconds. The same Anchor holds the room's Still organism in place. This joins
the source-placement puzzle and the creature rule instead of introducing a
separate switch. The other resonators accept direct illumination while holding R.

The three resonators are in Choir (14), Sealed Lung (16) and The Weight (22).
Choir can be reached with the opening light verb, whereas Sealed Lung requires
Veil. The lower loop links Curtain through Underside (17), Quiet Pool (18),
Unlit Span (19), The Weight (22), Suspension (12) and Afterimage (11). Quiet Pool
is the deep sanctuary. The strongest stalking and scale encounters belong in
Long Gallery (13), Unlit Span (19), or The Weight (22), with their spawn positions
offset from gates and the resonator centre. The Anchor also gates one approach
to The Weight, making remote illumination useful late in the journey.

Old Passage (20) is optional and reconnects Curtain with the opening's Witness
room once Veil is understood. Blind Crown (21), reached above Choir, contains
the other secret. High Crossing (15) links the northern branch back toward
Sealed Lung. Seeds may add cross-links 5–8, 6–15, 7–13, and 10–18. Their gates
preserve progression while changing travel decisions and return routes.

All three resonators open the final membrane beside Hollow. It leads to Last
Chamber (23), physically returning the player to the neighbourhood of the
opening. The ending site is the centre of that chamber. It must remain inert
until all three resonators are active, independently of collision at the gate.

`WorldMap::validate` checks abstract reachability for the original light verbs,
Anchor, Veil and the completed resonators. It also rejects unrelated room/tunnel
intersections and tests that each gate spans solid rock at both ends. Generation
uses bounded retries and a safe fallback with fixed landmark positions. Tests cover 2,048 seeds,
extreme seeds, deterministic reproduction, meaningful topology variation,
clearance along every physical passage segment and attempts to skirt membrane ends.

The headless tests in `src/validation.rs` run the actual Bevy gameplay schedule,
including input, movement, mechanisms, creatures and progression. They walk the
whole authored route on seeds 0, 72419 and 20260906, acquire both abilities,
illuminate all three resonators, return to Hollow and complete the ending.
Other checks establish that darkness cannot skip the opening light lesson,
Anchor must be placed within 180 units and latches only after crossing, Veil
requires extinguishing, creature rules differ, and death preserves progress.
Adversarial checks also exercise long browser frames against closed gates,
a dark turn that a Listener cannot track, Still organisms illuminated by a
remote Anchor, and the folded resonator's rejection of continuous direct light.
They complement browser playtesting; they do not evaluate atmosphere or fun.
Run all tests with `cargo test`. During concurrent browser development, use
`cargo test --target-dir target/native` to avoid locking the WASM build directory.

Keep rooms 0, 1, 3, 6, 7, 10, 18 and 23 free of predatory spawns. Rooms 0/1
teach before punishment; 6/10 teach new verbs; 3/7/18 provide pacing and save
landmarks. Prefer fewer creatures whose behaviour can be observed and heard
across passage mouths over placing a hostile creature in every room.

## Physical cover revision

Each chamber now contains seeded solid outcrops outside guarded entrance-to-site
routes. These follow the actual bent waypoints. Outcrops are smooth capsule forms in the same distance field used by movement,
visibility and lighting. Main-route clearance is validated across seeds. Creature
spawns choose a clear position instead of assuming a fixed offset is empty.
Gate throats have half-widths of roughly 58–74 world units; approaches change width and turn out of sight. The shared camera and pointer projection use a 390-unit view height. Existing save topology and
progression roles are preserved.

## Pursuit and recovery

Listeners pause to signal detection before pursuing at 66 units/second, below the traveller’s normal 82. They investigate the last detected location for 4.8 seconds and abandon a pursuit beyond 460 units from home. Still organisms move at 49 while investigating, freeze under visible light and hear nearby movement. Closed membranes obstruct sensing as well as movement; pulses cannot summon creatures through solid rock. The deep shadow has a slower, 27-unit searching front. Its seven tapered branches share geometry between contact detection and rendering, reach from beyond the cavern, swallow placed anchors and recede after The Weight is awakened.

First contact produces a recorded impact, a stumble and 1.5 seconds to escape. A second contact before the lantern recovers can still extinguish the player. Completed resonators are safe near their centre and become checkpoints. Rounded cover, protected approaches, swept tangent movement and recovery from an invalid saved rock contact reduce snagging without allowing traversal through closed gates. An open membrane waits until the player leaves its throat before closing.


## Regional rules and geography revision

Landmarks now occupy uneven, oblique circuits rather than uniform rows. The
opening remains guarded. The same progression graph governs concave shelves,
rifts and galleries, but the approaches change direction and spacing. Validation
rejects crossings and bypasses. Existing saves first validate against the legacy
world, then migrate checkpoints and latched passages by landmark/passage identity;
abilities, activated structures, secrets and deaths remain intact.

Ash chambers establish pulse/extinction rules over rough stratified ground.
Rootwork uses dark loam and tangled timber: root patches slow an extinguished
traveller, while personal light, a pulse or a nearby anchor releases their grip.
Stone galleries use pale fractured slabs and ruined arches; ordinary footfalls
carry farther to Listeners, while extinguished steps do not create that signal.
Silt reaches have subdued disturbed surfaces and quiet sand contacts; the
Span's shadow consumes the light used to understand and cross it.

Each barrier family has a distinct physical form and matching memory glyph:
irises accept light, thorns yield to darkness, paired ribs accept an anchor,
woven curtains require extinguishing with Veil, and the final structure has three
sockets. The Hollow repeats those sockets. Ability acquisition freezes danger
until Enter acknowledges a direct mechanical explanation. Pause and memory show
the current objective and earned verbs; completed structures send light back to
the Hollow, and progress survives death.

Corridors inherit their endpoint regions. The nearby centre of an unrelated room
behind rock cannot rename a passage or change its environmental rule.

## Finds and physical lanterns

The Fold holds a reusable bell. R collects it, Q throws it up to 168 units in the facing direction, and R retrieves it after landing. Solid geometry and closed passages stop the throw. Its recorded bell cue attracts investigating organisms independently of illumination; another cannot be thrown while it is on the ground.

Route stones in Hollow, Split Nerve and Afterimage mark Choir, Sealed Lung and The Weight respectively in memory. They do not reveal hidden creatures or all surrounding floor. Seven optional oil flasks occupy clear approaches in side spaces, with capacity two. Oil extends the next successfully placed lantern to one minute; cancelled placements consume nothing. Finds and carried items survive saves and deaths. Earlier saves default to empty pockets without changing progress or geography.

Lantern placement takes 1.15 seconds of stillness. Moving or being hurt cancels it; the old light remains until the replacement is ready. An ordinary lantern burns for 28 seconds. Creatures in contact need 1.4 seconds to snuff it. Oil use is saved immediately, preventing reload duplication. A dark shell remains after the flame expires.

Tests cover interrupted/replaced/expired lanterns, oil consumption, delayed creature snuffing, bell acquisition/throw/retrieval and hearing, route markers, old-save defaults, and clear approaches to all eleven finds across 128 additional seeds. The main seed suite and actual-system campaign routes also exercise the new placement and hold-to-interact rules.
