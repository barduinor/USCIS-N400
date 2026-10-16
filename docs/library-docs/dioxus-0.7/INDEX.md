# Dioxus 0.7 Task Index

Use this file to select a small Dioxus documentation target.
Open only the sections relevant to the task.

## Common Tasks

| Task | Read | Why |
| --- | --- | --- |
| Create or change components | `components-rsx.md`; Dioxus 0.7 docs: Building User Interfaces -> Components; Introducing RSX | Components are Rust functions that return `Element`; RSX is the supported UI syntax. |
| Change markup, attributes, or lists | `components-rsx.md`; Dioxus 0.7 docs: Elements and Text; Dynamic Attributes; Rendering Lists; Conditional Rendering | RSX has Dioxus-specific rules for text, attributes, conditionals, and keyed lists. |
| Add or change local reactive state | `signals-state.md`; Dioxus 0.7 docs: The Basics of State -> Storing State in Hooks; Reactive Signals | Dioxus 0.7 centers local UI reactivity on hooks and signals. |
| Handle user input | Dioxus 0.7 docs: The Basics of State -> User Input | Input events and controlled values use Dioxus event patterns. |
| Fetch async data | `async-data.md`; Dioxus 0.7 docs: The Basics of State -> Async and Futures; Data Fetching | Dioxus resources and futures are the documented async UI patterns. |
| Add effects or derived values | `async-data.md`; Dioxus 0.7 docs: The Basics of State -> Effects and Memos | Effects and memos have Dioxus-specific reactivity constraints. |
| Introduce routing | Dioxus 0.7 docs: Routing -> Defining Routes; Navigation; Layouts | Dioxus router docs cover route definitions, navigation, and layouts. |
| Change assets or styling | Dioxus 0.7 docs: Building User Interfaces -> Assets; Styling; Guides -> Tailwind | Asset and styling behavior is target-dependent. |
| Change web target behavior | `web-targets.md`; Dioxus 0.7 docs: Guides -> Platform Support -> Web | Web-target setup and bundling have dedicated guidance. |
| Test or debug Dioxus behavior | `anti-patterns.md`; Dioxus 0.7 docs: Guides -> Testing and Debugging -> Web Testing; Anti-patterns; Optimizing | Dioxus-specific debugging and anti-pattern docs cover common failure modes. |
| Migrate Dioxus versions | Dioxus 0.7 docs: Migration -> To 0.7 | Dioxus version migrations have dedicated upstream guidance. |

## Local Topic Notes

- `components-rsx.md` - components, RSX, lists, attributes, and markup.
- `signals-state.md` - signals, reactive reads, local UI state, and input
  state.
- `async-data.md` - resources, async handlers, effects, and suspense.
- `web-targets.md` - web target setup and bundling.
- `anti-patterns.md` - Dioxus-specific watch-outs and when to consult upstream
  anti-pattern guidance.

## Additional Exact Upstream URLs

Use these URLs for topics that do not yet have a dedicated local topic note:

- Routing -> Defining Routes:
  `https://dioxuslabs.com/learn/0.7/essentials/router/routes/`
- Routing -> Navigation:
  `https://dioxuslabs.com/learn/0.7/essentials/router/navigation/`
- Routing -> Layouts:
  `https://dioxuslabs.com/learn/0.7/essentials/router/layouts/`
- Building User Interfaces -> Assets:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/assets/`
- Building User Interfaces -> Styling:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/styling/`
- Guides -> Utilities -> Tailwind:
  `https://dioxuslabs.com/learn/0.7/guides/utilities/tailwind/`
- Migration -> To 0.7:
  `https://dioxuslabs.com/learn/0.7/migration/to_07/`

## Context Control

Do not load all upstream docs by default:

1. Identify the Dioxus concern from the task.
2. Open only the matching rows from this index.
3. Escalate to the full upstream source list in `SOURCES.md` only if the task is
   blocked by missing Dioxus detail.
