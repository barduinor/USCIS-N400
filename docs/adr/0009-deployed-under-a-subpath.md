# The app is served from a project subpath, and that path is the worker's scope

The deployed app lives at `https://barduinor.github.io/USCIS-N400/` — a GitHub Pages project site, served from a subpath rather than a domain root.

The subpath is not only where the files sit. It is the service worker's registration scope, the manifest's `scope`, and the base the app's assets resolve against. Every one of those has to agree, or the worker fails to register, or the manifest's `start_url` opens something other than the app, or an asset 404s.

## Considered Options

- **A user site at the domain root** would have removed the subpath entirely, at the cost of the repository being named `<owner>.github.io` and holding nothing else.
- **A custom domain** buys a stable name and a root path, and costs a DNS record and a certificate to keep alive. The project site was chosen for having no moving parts; a custom hostname stays available later for the same build.

## Consequences

- The base path belongs on the build command, not in `Dioxus.toml`, so local development keeps serving from `/`.
- Nothing in the shell may hard-code the path: the worker derives its cache URLs from its own registration scope, and the index template resolves the app root from the document's directory, so the same build works under any prefix.
- The base path is load-bearing for offline behaviour, which means a change to it is not a deployment detail — it invalidates what installed clients have cached.
- GitHub Pages answers an unknown path with `404.html`. The release copies the built shell there, so a deep link boots the app before the worker controls it. The worker itself fetches the root for every navigation and caches that response, so a deep-link refresh never runs an older shell than the one deployed; the cached root answers only when the network cannot.
