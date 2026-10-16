# Styling is plain CSS, not Tailwind

The app's styling is one `styles.css` linked from the index template, with CSS custom properties as the token set. No CSS framework, and no build step for styles.

This was decided while the app shell was built and, until now, only recorded in `docs/open-work.md` as the rationale beside a bullet that has since been marked built — where it would have been lost.

## Considered Options

- **Tailwind, as the Dioxus ecosystem defaults to it.** Rejected. It puts a Node or binary build step inside the pre-commit hook and CI, for seven screens of single-user UI where the stylesheet is a few hundred lines and changes rarely. The hook runs on every commit, so the cost lands on the loop that runs most often.
- **A component library.** Rejected for the same reason plus the vocabulary mismatch: the screens are text, a form, and figures, not an application shell.

## Consequences

- Class names are strings in the RSX, so adopting Tailwind later is a change to markup and styling only, with no data or behaviour to migrate.
- The token set lives in custom properties at the top of the stylesheet; a theme change is a change there.
- Phone-first layout rules are written by hand, so there is no framework's breakpoints to lean on — the sizes that matter are in the Dioxus standards.
