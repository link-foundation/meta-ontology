# 06 — Web Application (M4)

Goal: a static SPA on GitHub Pages where users can browse the ontology, see
how concepts define each other, and follow cycles.

Status: **planned for M4** — not implemented in PR #2.

## Stack (recommended for 2026)

- **Rust → WASM**: `wasm-pack` + `wasm-bindgen`.
- **Front‑end**: React 18+, written in TypeScript.
- **Bundler**: Vite + `@vitejs/plugin-react` + `vite-plugin-wasm` +
  `vite-plugin-top-level-await`.
- **Graph viz**: `react-cytoscapejs` (preferred; battle‑tested for thousands of
  nodes) or `reactflow` (better DX, less ideal for dense graphs).
- **State**: React Query for data, Zustand for UI state.
- **Styling**: Tailwind CSS.

## Repository layout

```
crates/
└── meta-ontology-wasm/
    ├── Cargo.toml          # crate-type = ["cdylib"]
    └── src/
        └── lib.rs          # wasm_bindgen exports

web/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── public/
│   └── data/               # built copies of data/*.lino (or fetched at runtime)
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── components/
    │   ├── ConceptGraph.tsx
    │   ├── ConceptDetail.tsx
    │   ├── LangSwitcher.tsx
    │   └── SearchBar.tsx
    └── lib/
        └── ontology.ts     # thin wrapper around the WASM module
```

## WASM crate (sketch)

```rust
use wasm_bindgen::prelude::*;
use meta_ontology::Ontology;

#[wasm_bindgen]
pub struct WasmOntology(Ontology);

#[wasm_bindgen]
impl WasmOntology {
    #[wasm_bindgen(constructor)]
    pub fn new(lino_text: &str) -> Result<WasmOntology, JsError> { /* ... */ }

    pub fn names(&self) -> Vec<JsValue> { /* ... */ }
    pub fn show(&self, name: &str) -> JsValue { /* serde_wasm_bindgen */ }
    pub fn neighbors(&self, name: &str) -> JsValue { /* ... */ }
}
```

## Build pipeline

1. `wasm-pack build crates/meta-ontology-wasm --target web`.
2. `cd web && pnpm install && pnpm build` — Vite produces `web/dist/`.
3. Deploy `web/dist/` to `gh-pages` branch under `app/` (the existing
   `cargo doc` deploy continues to publish to the root).

## GitHub Pages layout

```
https://<org>.github.io/meta-ontology/         ← cargo doc (existing)
https://<org>.github.io/meta-ontology/app/     ← React SPA (new)
```

Both can coexist via subdirectory deployment with
`peaceiris/actions-gh-pages@v4` and the `keep_files: true` option (or by
deploying both in a single step into separate folders of the same artefact).

## Features

### MVP (M4 first cut)

- Graph view with all concepts.
- Click a node → side panel with definitions, mappings, exponents.
- Search by name (substring).
- Highlight cycles (edges that participate in a cycle get a colour).
- Mobile responsive; basic dark mode.

### Polish (M5)

- Language switcher: pick one of the 20 supported languages → labels
  re‑render.
- "Random walk" mode: animate a random path through the graph.
- Embed mode: a query‑string flag that hides chrome, suitable for iframing.
- Permalinks to specific concepts.

## Performance budget

- Initial load: `<300 KiB` gzipped JS + `<200 KiB` WASM.
- Time to first interaction: `<2s` on a 4G connection.

## Accessibility

- Keyboard navigation through the concept list.
- High‑contrast theme.
- ARIA labels on graph nodes (cytoscape supports this via `aria-label` on
  the underlying canvas overlay).
