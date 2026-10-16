# Dioxus 0.7 Web Target

Read upstream Dioxus 0.7 docs first when changing web target behavior:

- Guides -> Platform Support -> Web:
  `https://dioxuslabs.com/learn/0.7/guides/platforms/web/`
- Guides -> Tools -> Configure Project:
  `https://dioxuslabs.com/learn/0.7/guides/tools/configure/`
- Guides -> Publishing -> Bundle Config:
  `https://dioxuslabs.com/learn/0.7/guides/deploy/config/`

## Dioxus Concepts

Dioxus supports web, desktop, and mobile targets, each with its own setup,
platform APIs, and bundling behavior. This pack covers the web target.

## Watch-Outs

- Keep web-platform APIs behind explicit configuration or `cfg` boundaries
  where they touch shared code.
- Prefer the 0.7 web guides over older or other-target examples.
