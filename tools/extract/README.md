# Extract step (Windows, PowerShell)

These scripts turn your own installed copy of Tricky Towers into the gitignored `extracted/`
folder the game loads (see `docs/art-layout.md`). They read only; nothing from the game is
committed. They contain names and measured positions, no code or assets from the original.

1. Export your install with AssetRipper (GUI or `--headless`, "Export Unity project") into
   `research\ripped` (gitignored).
2. Index asset GUIDs:
   `powershell -File tools\extract\guid_index.ps1 -Assets research\ripped\ExportedProject\Assets -Out research\guid_index.tsv`
3. Stage the named art, sky/water gradients, backgrounds, logo and the composed title backdrop:
   `powershell -File tools\extract\stage.ps1 -Repo .`
4. Write `extracted\inventory.md` and stage everything else under suggested names:
   `powershell -File tools\extract\inventory.ps1 -Repo .`

`prefab_sprites.ps1 -Prefab <file> -GuidIndex research\guid_index.tsv` lists which sprites a
prefab uses and where, for measuring new layouts.

Delete `extracted\` before re-running step 3 so stale entries don't linger.
