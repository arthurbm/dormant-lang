# Presets and your own colours

## Presets

A **preset** is a named set of colours for diagrams. Choose one per deck (`pele: { theme: flexoki }` in the headmatter) or per block (`{theme: 'flexoki'}`); [options.md](options.md) lists the Mermaid theme names that map to them.

- `auto` (the default) follows the Slidev theme: text and lines take the slide's text colour, fills are tints of it, the accent is the theme's primary colour (`--slidev-theme-primary`), the background is the one painted behind the diagram (the slide's, from the Slidev theme, the deck's CSS or the slide's `background`), and labels use the slide's font. Dark mode recolours it live.
- `flexoki` uses Flexoki's colours, light and dark, and draws the diagram on a card of its own background.

## Dark mode

Both presets follow Slidev's dark mode (the `d` key, or `colorSchema` in the headmatter) without drawing the diagram again. To force one diagram dark whatever the mode, use `{theme: 'dark'}`.

## Your own colours

Pele draws every colour from `--pele-*` CSS variables. Set them on `.pele-diagram`, in the deck's `style.css` or a slide's `<style>`; any rule of yours wins over the preset, and dark mode is an `html.dark` rule:

```css
.pele-diagram {
  --pele-accent: tomato;
  --pele-font: "IBM Plex Sans";
}
html.dark .pele-diagram {
  --pele-bg: #000;
}
```

Setting them on `:root` has no effect: the preset sets them on `.pele-diagram`, which is closer.

| variable                              | colours                                    |
| ------------------------------------- | ------------------------------------------ |
| `--pele-bg`                           | background, and text on some shapes        |
| `--pele-fg`                           | text                                       |
| `--pele-muted`                        | secondary text                             |
| `--pele-line`                         | edges and arrowheads                       |
| `--pele-surface`                      | node fill                                  |
| `--pele-surface-alt`                  | subgraph fill                              |
| `--pele-border`                       | node and subgraph borders                  |
| `--pele-accent`                       | highlights                                 |
| `--pele-critical`                     | gantt critical tasks, kanban high priority |
| `--pele-error`                        | error text                                 |
| `--pele-font`, `--pele-font-mono`     | label fonts                                |
| `--pele-radius`                       | corner radius                              |
| `--pele-series-1` … `--pele-series-8` | chart series (pie, xychart, radar…)        |

Mermaid's `style` and `classDef` lines still apply, with fixed colours that do not follow the preset or dark mode.

## Restyling steps

Each part of a stepping diagram carries one class at each click: `pele-step-active`, `pele-step-receding` (focus mode, drawn at 20% opacity) or `pele-step-hidden` (reveal mode). To change how a step looks, style those classes under `.pele-diagram.pele-steps` in the deck's `style.css`:

```css
.pele-diagram.pele-steps .pele-step-receding {
  opacity: 0.4;
}
```

In a slide's scoped `<style>`, reach the drawing with `:deep()`: `.pele-diagram :deep(.pele-step-receding) { opacity: 0.4; }`.
