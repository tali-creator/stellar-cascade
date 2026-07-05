# Setup notes

## What was actually broken

The previous pass referenced two Tailwind classes that don't exist anywhere —
`animate-marquee` and `animate-flow` — without ever defining them. Tailwind
doesn't know about them, so it silently drops them and nothing moves. Same
root cause on the fonts: `font-['IBM_Plex_Mono']` only sets `font-family` in
CSS, it doesn't fetch the font file, so it was falling back to a system font
the whole time.

Fixed with two small real files instead of editing `tailwind.config.ts`:

## 1. `components/animations.css`

Plain CSS `@keyframes`, imported directly into the two components that use
them:

```css
@keyframes marquee-scroll { from { transform: translateX(0); } to { transform: translateX(-50%); } }
.marquee-track { animation: marquee-scroll 24s linear infinite; }

@keyframes flow-dash { to { stroke-dashoffset: -56; } }
.flow-dash { stroke-dasharray: 6 8; animation: flow-dash 1.6s linear infinite; }
```

- `Marquee.tsx` imports it and uses `className="marquee-track ..."` on the scrolling track.
- `CascadeDiagram.tsx` imports it and uses `className="flow-dash"` on the four
  connector `<path>` elements that should show flowing dashes (the greyed-out
  "unclaimed" branch to `test-runner` intentionally has no class — it's meant
  to look inert).

No `tailwind.config.ts` edit needed at all — plain CSS handles what Tailwind
utilities can't express inline.

## 2. `app/layout.tsx`

Loads IBM Plex Mono / IBM Plex Sans by their literal family name via a Google
Fonts `<link>`, because components already reference the font that way
(`font-['IBM_Plex_Mono']`). Using `next/font` instead would generate a hashed
family name that wouldn't match those existing classes across all 8
components — the `<link>` approach means zero changes to any component file.

**This is provided as a reference file, not a blind overwrite** — you already
have a real `app/layout.tsx` and `globals.css` in your project. Merge in just
the two `<link>` tags inside `<head>` and add `font-['IBM_Plex_Sans']` (or
your own font utility) to your existing `<body>` className.

## File map

```
app/
  layout.tsx            — reference: font <link> tags to merge into your real layout
  page.tsx              — assembles all sections
components/
  animations.css        — the two @keyframes + animation classes
  Header.tsx            — fixed nav bar
  Hero.tsx              — headline, sub-copy, CTAs, mounts CascadeDiagram
  CascadeDiagram.tsx    — animated dependency-tree SVG (imports animations.css)
  Marquee.tsx           — scrolling spec ticker (imports animations.css)
  HowItWorks.tsx        — 3-step process grid
  FundingModes.tsx      — Stream vs Give comparison cards
  WhySoroban.tsx        — constraint comparison table
  Cta.tsx               — closing call to action
  Footer.tsx            — footer links + brand
```

Drop the `components/` folder alongside your existing ones. `app/page.tsx` is
ready to use as-is or paste into your existing page.
