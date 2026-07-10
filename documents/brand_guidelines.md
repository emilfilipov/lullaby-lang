# Lullaby Brand Guidelines

The visual identity for **Lullaby**, the compiled systems language. One system,
applied at the fidelity each surface allows: full-colour in the offline docs and
web, ANSI-approximated in the CLI, and a filled icon for OS/app/installer contexts.

> Personality: **warm & friendly**. Lullaby is a serious, fast, memory-safe systems
> language with a calm, gentle surface. The brand leans into that contrast — soft to
> meet, solid underneath.

Canonical assets live in [`assets/brand/`](../assets/brand/). Do not re-draw the
mark by hand; reuse these files or regenerate the raster icons with
`python assets/brand/render_icons.py`.

## The mark

An **"L" cradling a crescent moon** — the letter's corner becomes a cradle and the
moon rests in it. One motif carries the whole idea (a lullaby: moon + cradle) and it
stays legible from a hero down to a 16 px favicon.

- **Primary — one-ink lavender.** [`lullaby-mark.svg`](../assets/brand/lullaby-mark.svg),
  drawn in **lavender `#8b6ff0`**. Use everywhere: docs headers, the web, inline, and
  the CLI (as a Unicode/ANSI approximation with the 🌙 glyph). Recolour to plum ink
  `#372a54` only where a mono, single-ground lockup is needed.
- **Filled icon — the small-size exception.** [`lullaby-icon.svg`](../assets/brand/lullaby-icon.svg)
  and [`lullaby.ico`](../assets/brand/lullaby.ico): the mark in **cream** on a soft
  lavender→sky tile. Use **only** for the app/file icon, favicon, taskbar, and the
  installer, where a bare line-mark would look faint. Everywhere else stays one-ink.

Clear space: keep at least the height of the crescent clear on all sides. Don't
recolour the mark outside the palette, rotate it, add effects, or stretch it.

## Wordmark

"**lullaby**" — always lowercase, set in **Nunito ExtraBold (800)**, tinted the same
lavender as the mark (`#8b6ff0`; `#c9bafd` on dark grounds). Pair it with the mark in
a horizontal or stacked lockup, or use the mark alone in tight spaces.

## Palette

| Token | Hex | Role |
| --- | --- | --- |
| Lavender | `#c4b5fd` | Primary accent (mark ink: `#8b6ff0` light / `#c9bafd` dark) |
| Peach | `#fecaca` | Warm accent |
| Sky | `#bae6fd` | Cool accent |
| Moonglow | `#ffdca6` | Highlight |
| Cream | `#fff8ef` | Light ground / mark-on-tile |
| Plum ink | `#372a54` | Text; also the dusk ground in dark mode |

Neutrals are never pure black — text is the plum ink so the system stays warm. Dark
mode grounds on dusk plum (`#161020`–`#221a31`); the pastels stay luminous on it.

## Typography

- **Nunito** — one rounded humanist family for everything a person reads (display,
  UI, body). Weights: 400 / 600 / 700 / 800. Bundled with the toolchain
  ([`nunito.woff2`](../assets/brand/nunito.woff2), OFL) and embedded in the offline
  docs so they render identically offline, on any machine.
- **Monospace** for code: Cascadia Code / JetBrains Mono / system mono fallback.

## Voice & tagline

Friendly, plain, and reassuring. Say what happens; no jargon where a plain word will
do; errors explain the fix. A gentle bedtime metaphor is welcome but never at the
cost of clarity.

- **Tagline:** *Serious systems code. Sweet dreams.*
- Sign-off used by the CLI/first-run: *sleep easy — it's memory-safe.*

## Asset inventory

| File | Use |
| --- | --- |
| `assets/brand/lullaby-mark.svg` | Primary one-ink lavender mark |
| `assets/brand/lullaby-icon.svg` | Filled tile icon (app/favicon/installer) |
| `assets/brand/lullaby.ico` | Multi-size icon 16–256 (exe, installer, favicon) |
| `assets/brand/lullaby-icon-256.png`, `-512.png` | Raster app icon |
| `assets/brand/nunito.woff2` | Bundled body/display typeface (OFL) |
| `assets/brand/render_icons.py` | Regenerates the raster icons from the geometry |

The colour tokens, tagline, and lockups are previewed in the visual-identity board
(shared separately); this document is the source of truth for hex values and usage.
