# The app is client-side only, with no server

A single-user, phone-first Dioxus/WASM application that runs entirely in the
browser and is served as static files from GitHub Pages. All state (Profile,
Answer Log, exam history) lives in IndexedDB on the device. There is no
backend.

## Considered Options

- **Server plus database.** Rejected. A backend for one user adds hosting, auth,
  and sync for no benefit, and the app has to work offline on a phone.
- **Hosted key-value sync.** Rejected for v1. Still needs network, still needs
  auth, still breaks offline, for one user who owns every device involved.

## Consequences

- Progress is device-local. Clearing browser data destroys it, and iOS may evict
  storage for sites not installed to the home screen.
- **Export/import of progress as JSON is required in v1.** It is the only backup
  and the only way to move between phone and desktop.
- Install as a PWA with a service worker, so it works offline and its storage is
  durable.
- A Rust/WASM bundle runs ~1–3 MB against ~100 KB for plain JS. Irrelevant once
  the service worker caches it; the first load on cellular costs a couple of
  seconds.
