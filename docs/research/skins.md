# Wizard skins and brick packs in the original (read from the PC install, 2026-10-05)

## Wizards (characters)
The original has 14 characters, all built the same way (body, face, arms, cape…, with the same
set of clips: Idle, Action, Cheer, CastSelf, Casting, Laugh, Startle, Dying, plus CastStart,
CastEnd and LaughHard).

| Ours | Original prefab | How you get it |
|---|---|---|
| wizard1–4 | Wizard01–04 | Always available |
| wizard5–8 | WizardAnimal01–04 | Character DLC pack 1 (Steam app 481360) |
| wizard9 | WizardIndie01 | Indie pack (Steam app 667750), from Lethal League |
| wizard10 | WizardIndie02 | Indie pack, from Awesomenauts |
| wizard11 | WizardIndie03 | Indie pack, from TowerFall |
| wizard12 | WizardIndie04 | Indie pack, from Armello |
| wizard13 | WizardIndie05 | Indie pack, from Nuclear Throne |
| wizard14 | WizardIndie06 | Indie pack, from Octodad |

- There's no XP or level unlock: a character is available when the base game or its DLC is
  owned. Each has a player-overview avatar (WizardAvatar01–14), and the Indie ones show their
  game's logo (`gph_logo_<game>`) on their Select Players card.
- All 14 are now staged by `tools/extract/wizards.ps1` as `wizard<n>_<part>` with their clips
  (`wizard<n>_<part>_<clip>`), exactly like wizards 1–4 (816 entries). Voice lines still use
  the shared pool.

## Brick packs (brick skins)
Each player picks a brick pack; it changes the art of that player's normal bricks only (the
seven shapes). Curse looks (stone, ice, mystery) keep their own art.

| Pack | How you get it | What it looks like |
|---|---|---|
| Default | Always | `brick_<shape>` |
| Christmas | Always (free) | Own textures (`brick_christmas_<shape>`), a blinking lights layer (`brick_christmas_lights_<shape>`) with a random time offset per brick, and snow caps on top edges (`brick_christmas_snow_1..3`, 25% chance per edge, appearing within 0.3 s). Own placing sounds (201_Bricks_christmas). |
| Spring (Easter) | Always (free) | Own textures (`brick_spring_<shape>`), grass on edges (25% chance per edge) and egg or flower decorations (40% chance) that pulse to 0.8× every 5–15 s. Own placing sounds (206_Bricks_spring). |
| Halloween | Always (free) | Own textures (`brick_halloween_<shape>`), up to 2 front decorations per brick (bat, spider, pumpkin, cobweb; 40% chance each) that pulse every 5–15 s, and a black-and-white look. |
| Candy | DLC (app 570361) | Own textures (`brick_candy_<shape>`). |
| Hologram | DLC (app 570362) | Default textures through a hologram shader that glitches (distorts) for 0.1–0.5 s every 6–15 s. |
| Diamond | DLC (app 570363) | Own textures (`brick_diamond_<shape>`) with a gloss sweep at a random angle every few seconds (dark green border, two gloss colours). |
| Galaxy | DLC (app 570360) | No texture: every brick shows the same live starfield (`brick_galaxy_background` scrolling, `brick_galaxy_star` stars, one twirls every 2–6 s) inside a dark purple border (0.12, 0.03, 0.24). |

- Each pack also has its own next-brick card icons (`next_brick_<pack>_<shape>`) and a pack icon
  (`brick_pack_icon_<pack>`) for the picker.
- All of the above art is staged (421 stage entries, 0 missing).

## Where they're picked
- On **Select Players**, each joined seat has two horizontal pickers: a **Character** panel
  (left/right through the available wizards; the picked one cheers with a voice line) and a
  **Brick pack** panel under it (left/right through the available packs; picking plays that
  pack's brick-placed sound). Both are per seat, so every player has their own wizard and
  bricks.
- Only owned packs and characters are listed (the free seasonal packs always are).