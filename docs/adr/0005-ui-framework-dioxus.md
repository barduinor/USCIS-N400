# The UI framework is Dioxus, pinned to 0.7.x

The app runs client-side only (ADR 0003): no server, no SSR, no hydration.
Dioxus treats the web as a first-class target — router, asset pipeline, and
hot reload arrive with the one `dx` tool — and its cross-platform reach costs
nothing while going unused. Pin `dioxus` to 0.7.x; 0.8 is in alpha.

## Considered Options

- **Leptos.** Rejected. The usual recommendation for web-first Rust, but its
  defining strengths are SSR, server functions, Islands, and SEO — all removed
  by ADR 0003. We would run it in a secondary CSR mode with a heavier build
  path (cargo-leptos or trunk).
- **Yew.** Rejected. Maintained, lower momentum, no advantage here.
- **A TypeScript stack (Svelte/Solid/Preact on Vite).** Rejected. The least
  friction for a small offline PWA in isolation, but it forks the app away
  from the `domain` crate, its native test loop, and the Rust standards. A
  hybrid TS UI over a wasm-built domain adds a second toolchain for one user.

## Consequences

- UI framework rules live in `docs/coding-standards/dioxus/`, not in the Rust
  entry point.
- Testable behavior belongs in `domain`; the UI layer stays thin.
- Bundle size is in the range ADR 0003 already accepted, and the service
  worker caches it after the first load.
