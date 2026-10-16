# Library Docs

This is the canonical entry point for curated library documentation used by
implementation agents.

Library docs should be project-agnostic. They may summarize upstream library
concepts, record source provenance, and point agents to relevant official
documentation. Project-specific architectural decisions belong in ADRs, and
repeatable project rules belong in coding standards.

## How To Navigate

Do not load this whole tree by default. Use this README as the library index,
then open only the folder that matches the task.

1. Identify the library or tool involved in the task.
2. Open that folder's README first.
3. If the folder has an `INDEX.md`, use it to select the narrow topic note or
   upstream source needed for the task.
4. If there is no `INDEX.md`, treat the folder README as the complete local
   navigation note for that library.
5. Escalate to upstream documentation only from the sources listed in the
   selected library folder.

## Available Docs

| Library docs | Start here | Use when |
| --- | --- | --- |
| `docs/library-docs/dioxus-0.7/` | `README.md`, then `INDEX.md` | Working on Dioxus 0.7 UI components, RSX, signals, async UI data, routing, assets, styling, or web target behavior. |

Add a library folder only when the project adopts or evaluates a library whose
upstream documentation needs curated local navigation or summaries. Keep local
policy out of this tree.
