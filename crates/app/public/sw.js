// The offline worker: the app shell and the assets it needed to boot, kept in
// one versioned cache (ADR 0003: no server, and the app has to work offline on
// a phone).
//
// Every path here is relative to the worker's own scope — the root in
// development, the repository subpath on Pages — so no base path is written
// down. The build also hashes the names of the files it emits, so no asset is
// named here either: the shell is the scope itself, and the rest is whatever
// the running app asked for. A hand-written precache list would name files no
// build produces.

// The caches this worker owns are named with this prefix and a version. Change
// the version when the shell's own files change: the next activation drops the
// cache the previous version built. A deploy that only re-emits the hashed
// bundles needs no bump, because the shell asks for them under new names.
const PREFIX = "uscis-n400-";
const VERSION = 1;
const CACHE = `${PREFIX}v${VERSION}`;

// The app root: the worker's own scope. The shell lives at it, and every route
// the app serves is under it.
const ROOT = new URL("./", self.registration.scope).href;

// Install: keep the shell, so the first load after this one works without the
// network, and take over as soon as the browser lets this worker run.
self.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      const response = await fetch(ROOT, { cache: "reload" });

      if (response.ok) {
        const cache = await caches.open(CACHE);
        await cache.put(ROOT, response);
      }
    })()
      .catch(() => {
        // The network was not there: the next navigation caches the shell.
      })
      .then(() => self.skipWaiting()),
  );
});

// Activate: drop every cache an earlier version built, and answer for the pages
// that are already open.
self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const names = await caches.keys();

      await Promise.all(
        names
          .filter((name) => name.startsWith(PREFIX) && name !== CACHE)
          .map((name) => caches.delete(name)),
      );

      await self.clients.claim();
    })(),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;

  // Anything outside the app is the browser's and the network's business.
  if (request.method !== "GET" || !request.url.startsWith(ROOT)) {
    return;
  }

  // A navigation is the shell. The host answers it when it can, so a deploy
  // arrives with the next load; the cached shell answers it when the host
  // cannot — offline, or a route the host has no file for.
  if (request.mode === "navigate") {
    event.respondWith(shell(event, request));
    return;
  }

  // Everything else is the app's own files: served from the cache, and fetched
  // again in the background so a change to a file the build does not hash
  // arrives with the next load.
  event.respondWith(kept(event, request));
});

// The page tells the worker which resources it needed to boot. They are kept
// under the worker's own scope, so the installed app opens offline after one
// online visit, and the page names them because only the build knows the hashed
// names.
self.addEventListener("message", (event) => {
  const resources = event.data && event.data.resources;

  if (!Array.isArray(resources)) {
    return;
  }

  event.waitUntil(keep(resources));
});

// The shell for a navigation: what the host serves, else what the cache kept.
async function shell(event, request) {
  let answer = null;

  try {
    answer = await fetch(request);
  } catch (error) {
    // Offline: the cached shell answers below.
  }

  if (answer && answer.ok) {
    event.waitUntil(
      caches.open(CACHE).then((cache) => cache.put(ROOT, answer.clone())),
    );

    return answer;
  }

  const cache = await caches.open(CACHE);
  const shell = await cache.match(ROOT);

  return shell || answer || Response.error();
}

// One of the app's own resources: what the cache holds, else the network.
async function kept(event, request) {
  const cache = await caches.open(CACHE);
  const answer = await cache.match(request, { ignoreVary: true });

  if (answer) {
    event.waitUntil(refresh(cache, request));

    return answer;
  }

  const response = await fetch(request);

  if (response.ok && response.type === "basic") {
    event.waitUntil(cache.put(request, response.clone()));
  }

  return response;
}

// Fetches one kept response again, so the next load has the deployed file.
async function refresh(cache, request) {
  try {
    const response = await fetch(request);

    if (response.ok && response.type === "basic") {
      await cache.put(request, response);
    }
  } catch (error) {
    // Offline: what the cache holds stays.
  }
}

// Keeps the resources the page booted from, skipping the ones already held.
async function keep(resources) {
  const cache = await caches.open(CACHE);

  await Promise.all(
    resources.map(async (url) => {
      if (typeof url !== "string" || !url.startsWith(ROOT)) {
        return;
      }

      if (await cache.match(url, { ignoreVary: true })) {
        return;
      }

      try {
        await cache.add(url);
      } catch (error) {
        // A resource the page did not get either: the app runs without it.
      }
    }),
  );
}
