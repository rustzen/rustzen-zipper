# Rustzen Zipper Brand Assets

These source assets align Rustzen Zipper with the Zen Blue Glass direction used by Rustzen Clear.

## Clear icon parity spec

Measured against the approved Rustzen Clear `icon-1024.png` reference:

- Canvas: `1024 x 1024`.
- Main card visual bounds: about `x=104 y=104 w=816 h=816`.
- Soft shadow may extend below the card to about `y=960`.
- Outer card radius target: `rx=168`.
- Inner rim target: `x=140 y=140 w=744 h=744 rx=140`.
- Main symbol should stay inside the central visual area and leave generous white-blue padding.
- Background must stay white / `#EEF4FE` / `#D3E7FD`; avoid full-surface saturated blue.

## Style direction

Rustzen Zipper should not use heavy dark-blue 3D or high-detail realistic zipper imagery. It should use the same calm tool aesthetic as Rustzen Clear:

- low-contrast blue-white glass surfaces
- soft translucent panels
- light shadows
- rounded geometry
- restrained zipper detail
- abstract `Z` silhouette rather than a literal object-heavy zipper

## Palette

| Token | Value | Usage |
| --- | --- | --- |
| `zen-blue-deep` | `#77A8F7` | primary accent |
| `zen-blue-mid` | `#A8CBFB` | secondary accent |
| `zen-blue-highlight` | `#D3E7FD` | highlight / soft fill |
| `zen-panel` | `#EEF4FE` | background / glass panel |
| `zen-shadow-edge` | `#DDE6F4` | border / separator |
| `text-primary` | `#2F3E58` | title text |
| `text-secondary` | `#60708A` | secondary text |

## Assets

- `logo-lockup.svg`: horizontal logo lockup.
- `menubar-icon.svg`: source menu bar glyph using `currentColor`.
- `menubar-variants.svg`: design sheet for light/dark/accent states.
- `menu-popover-concept.svg`: menu bar dropdown visual direction.

## Runtime app icon

Runtime app icons are committed directly under `apps/macos/src-tauri/icons/`.
Do not regenerate them from an SVG source unless a new reviewed source asset is
explicitly added.

Required runtime outputs are `32x32.png`, `128x128.png`, `128x128@2x.png`,
`icon.png`, `icon.icns`, and `icon.ico`.
