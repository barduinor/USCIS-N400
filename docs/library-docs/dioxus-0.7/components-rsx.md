# Dioxus 0.7 Components And RSX

Read upstream Dioxus 0.7 docs first when changing component structure:

- Building User Interfaces -> Components:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/components/`
- Building User Interfaces -> Introducing RSX:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/rsx/`
- Building User Interfaces -> Elements and Text:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/elements/`
- Building User Interfaces -> Dynamic Attributes:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/attributes/`
- Building User Interfaces -> Rendering Lists:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/iteration/`
- Building User Interfaces -> Conditional Rendering:
  `https://dioxuslabs.com/learn/0.7/essentials/ui/conditional/`

## Dioxus Concepts

Dioxus components are Rust functions that return `Element`. Components use
`rsx!` for markup, attributes, conditional rendering, and list rendering.

## Watch-Outs

- Prefer Dioxus RSX attributes and event syntax from the 0.7 docs over older
  examples from previous releases.
- For conditional rendering, make states explicit enough that loading, empty,
  error, and populated UI remain easy to inspect.
