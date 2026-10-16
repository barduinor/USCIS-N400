# Dioxus 0.7 Async Data

Read upstream Dioxus 0.7 docs first when changing async UI work:

- The Basics of State -> Async and Futures:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/async/`
- The Basics of State -> Data Fetching:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/resources/`
- The Basics of State -> Effects and Memos:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/effects/`
- The Basics of State -> Error Handling:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/error_handling/`
- The Basics of State -> Suspense:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/suspense/`

## Dioxus Concepts

Dioxus async UI work can use futures, resources, effects, memos, and suspense.
Resources rerun when their tracked reactive dependencies change.

## Watch-Outs

- Surface loading, empty, error, and success states distinctly in RSX.
- When forcing a reload, make the reactive dependency explicit enough that a
  future maintainer can see why the resource reruns.
