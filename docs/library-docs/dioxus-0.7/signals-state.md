# Dioxus 0.7 Signals And State

Read upstream Dioxus 0.7 docs first when changing local reactive state:

- The Basics of State -> Storing State in Hooks:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/hooks/`
- The Basics of State -> Reactive Signals:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/signals/`
- The Basics of State -> User Input:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/event_handlers/`
- The Basics of State -> Hoisting State:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/hoisting/`
- The Basics of State -> Global Context:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/context/`
- The Basics of State -> Stores and Collections:
  `https://dioxuslabs.com/learn/0.7/essentials/basics/collections/`

## Dioxus Concepts

Dioxus signals are reactive values. Components subscribe to signals when they
read them during render.

## Watch-Outs

- Use signals for local reactive UI state rather than plain mutable variables
  that RSX cannot observe.
- Read signals in render paths only when the component should subscribe to
  those values.
- Hoist state only when multiple components need the same live UI state.
- Be careful when passing writable signals to child components; the child then
  participates in mutating parent-owned state.
