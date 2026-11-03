# Options

A diagram block's options go in the braces after its steps (or alone): ` ```mermaid {a|b}{reveal: true, scale: 0.8} `. As in Slidev, the braces are a JavaScript object written into a double-quoted attribute, so quote strings with single quotes (`{theme: 'dark'}`); a double quote breaks the slide. Write literal values: the addon checks only those, and a `theme` computed in the slide is not read, so the deck's applies. Any other key gives a `[pele]` warning listing these. Steps and options each take their own braces: `{a|b, reveal: true}` in one pair is read as `{a|b}{reveal: true}`, with a warning saying to write it that way. A computed `scale` that is not a positive number is reported in the browser console with `[pele]`.

## `scale`

A number greater than 0. The diagram's height is its natural height times `scale`, as in Slidev's Mermaid blocks; it never grows wider than its column. Without it, the diagram draws at its natural size, shrunk to fit the column.

## `theme`

The preset for this block, overriding the deck's `pele.theme`: `'auto'` (the default) or `'flexoki'`, or one of Mermaid's theme names, mapped to a preset:

| `theme`               | draws with                                        |
| --------------------- | ------------------------------------------------- |
| `'default'`, `'base'` | `auto`                                            |
| `'neutral'`           | `auto` in greys                                   |
| `'dark'`              | `auto` forced dark, whatever Slidev's colour mode |
| `'forest'`            | `auto`, with a warning: Pele has no forest preset |

See [presets.md](presets.md) for what each preset looks like and how to restyle it.

## `reveal`

`true` switches the block's steps to reveal mode: parts stay hidden until their step arrives. Without it, steps use focus mode. On a block without steps it does nothing and gives a `[pele]` warning.

## `renderer`

`'mermaid'` hands one ` ```mermaid ` block to mermaid.js. `'pele'` draws a ` ```mermaid ` block with Pele even when the deck sets `pele: { mermaid: false }`. A ` ```pele ` block always draws with Pele and only warns on `renderer: 'mermaid'`. A handed-back block cannot step: its steps are dropped with a warning.

## `at`

The click the block's second step lands on, exactly as `at` on Slidev's code blocks. A number is absolute (`{at: 1}`: step 2 at click 1); a string with a sign is relative to what comes before on the slide (`'+2'`); the default is `'+1'`. Give a code block and a diagram the same `{at: 1}` to step them together. `{at: false}` takes no clicks and shows the last step. On a block without steps it does nothing and gives a `[pele]` warning.

# Deck settings

Under `pele` in the deck's headmatter (the first slide's frontmatter). Any other key gives a `[pele]` warning listing these.

```yaml
---
addons:
  - pele
pele:
  theme: flexoki
  mermaid: false
---
```

## `pele.theme`

The preset for every diagram block: any `theme` value above. A block's own `theme` wins over it.

## `pele.mermaid`

`false` draws every ` ```mermaid ` block with mermaid.js; ` ```pele ` blocks, and blocks with `{renderer: 'pele'}`, still draw with Pele. Any value other than `true` or `false` gives a warning.

# Magic-move options

Options on the outer ` ````md magic-move ` fence of diagram versions, as in Slidev: ` ````md magic-move {at: 2, duration: 1200} `. Slidev's `lines`, any other key, and a `[title]` give a `[pele]` warning and are ignored, since a diagram has no code lines or file name. Versions take no steps or options of their own.

## `at`

The click the second version lands on, as for Slidev's magic-move; `{at: false}` takes no clicks and shows the last version.

## `duration`

How long the glide to the next version takes, in milliseconds. The default is the headmatter's `magicMoveDuration`, as in Slidev.
