# Hook0 Documentation

## Diagrams — Mermaid Design System (dark/light mode)

- NEVER use ASCII art in documentation. Always use Mermaid diagrams.
- Docusaurus has `@docusaurus/theme-mermaid` configured with `{ light: 'base', dark: 'dark' }`.
- NEVER use `style NodeX fill:#hex` in mermaid blocks. Use `classDef` + `:::class` syntax instead.
- `classDef` lines in .md files set **light mode colors inline** (fill, stroke, color). This makes light mode work out of the box.
- **Dark mode** is handled by a JS client module (`documentation/src/mermaid/theme-switcher.js`) that mutates inline SVG styles on theme change. CSS `!important` cannot reliably override Mermaid's inline `style` attributes, so JS is required.
- CSS variables for mermaid colors are kept in `documentation/src/css/custom.css` for reference but are NOT used for node styling.
- Prefer `flowchart LR` for data flows, `stateDiagram-v2` for state machines.

### Semantic node classes

| Class | Semantic role | Light mode | Dark mode |
|-------|--------------|------------|-----------|
| `:::external` | External systems, inputs, API clients | Blue (#dbeafe bg) | Deep blue (#1e3a5f bg) |
| `:::hook0` | Hook0 components, infrastructure | Green (#dcfce7 bg) | Deep green (#052e16 bg) |
| `:::customer` | User/customer endpoints, outputs | Orange (#ffedd5 bg) | Deep orange (#431407 bg) |
| `:::processing` | Internal processing, middleware | Purple (#ede9fe bg) | Deep purple (#2e1065 bg) |
| `:::danger` | Error/failure states | Red (#fee2e2 bg) | Deep red (#450a0a bg) |

### Template for new mermaid blocks

```
flowchart LR
    A["Input"]:::external --> B["Hook0"]:::hook0
    B --> C["Output"]:::customer
    B -.- D["Internal"]:::processing

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764
    classDef danger fill:#fee2e2,stroke:#f87171,color:#7f1d1d
```

### Rules
1. ALWAYS add `classDef` lines at the bottom of the mermaid block (before `click` lines).
2. ALWAYS apply `:::class` to every node definition.
3. The `classDef` values are the light-mode colors. The JS client module (`src/mermaid/theme-switcher.js`) handles dark mode by mutating inline styles.
4. For subgraph-level styling, use `class SubgraphName hook0` syntax.
5. Only include the `classDef` lines for classes actually used in the diagram.
