# Rust Coding Standards

This is the entry point for Rust-specific coding standards.

Use the global standards in `docs/coding-standards/README.md` first. In
particular, the global test integrity and naming rules still apply to all Rust
work. The files in `rust-best-practices/` add Rust-specific detail; they do not
replace the global rules.

## Interpretation

- Use the established domain vocabulary of the project in Rust names, tests,
  errors, API payloads, and comments. Prefer names built from domain terms over
  generic utility, storage, or status language.
- Keep Rust code focused on the boundary being changed. UI, backend,
  repository, migration, and test-support code should each carry their own
  intent instead of sharing vague utility names.
- Treat imported Rust10x examples as patterns to adapt, not vocabulary to copy.
  If an example mentions another domain, translate the structure into this
  project's domain terms.
- Treat the Rust10x pack as imported guidance for agents. The project-owned
  standards in this README decide how that guidance applies to a given project.
- Do not add UI framework rules here. Framework guidance lives in its own
  entry point: `docs/coding-standards/dioxus/README.md`.

## Core Rust Rules

Apply these rules to normal Rust implementation work:

- Use Rust Edition 2024 conventions when adding or updating crates unless the
  existing crate explicitly uses another edition.
- Avoid `.unwrap()` and `.expect(...)` in application, example, and test code.
  Prefer `?`, explicit error conversion, or `.ok_or("...")?` in tests and
  examples.
- Use `.unwrap_or...` variants when they express real fallback behavior.
- Prefer `Default` for empty infallible constructors, `new(...)` for obvious
  required arguments, `from_...(...)` for secondary constructors, and
  `with_...(...)` for self-consuming fluent configuration.
- Use a separate builder only when construction is complex, fallible, or needs
  validation.
- Prefer modern Rust patterns such as if-let chains and concise standard
  library helpers over manual nested matching.
- Import macros with `use ...` and invoke the local macro name instead of using
  fully qualified macro calls throughout the code.
- Keep `src/main.rs` and other Rust binary entry points limited to startup
  orchestration. They may parse or receive launch configuration, initialize
  runtime dependencies, and call into application modules, but durable behavior
  belongs in named modules.
- Extract cohesive responsibilities from Rust entry points before adding more
  behavior to them. UI trees, backend handlers, repository logic, domain
  workflows, and reusable helpers should live in modules named for their
  responsibility rather than in `main.rs`.
- For Rust UI code, extract a component into a named module or file when it has
  its own state, async resource, mutation workflow, or testable rendering
  behavior.
- Organize larger files with public types first, public implementations next,
  private support code after that, and tests at the end.
- Use code regions sparingly for large, meaningful sections such as `Modules`,
  `Types`, `Support`, `Iterator Implementations`, and `Tests`.

## Testing Rules

The global test integrity rules are authoritative. Rust-specific test guidance
adds structure:

- Use behavior-focused test names in the form
  `test_[module_path]_[function_name]_[variant]`.
- In Rust test modules, use
  `type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;`
  when fallible setup makes `?` clearer than repeated assertions.
- Structure larger tests with `// -- Setup & Fixtures`, `// -- Exec`, and
  `// -- Check` comments when the sections make the behavior easier to scan.
- Put large inline unit test modules behind a dedicated `_tests.rs` file or
  `src/_tests/...` file only when the source file becomes hard to read.
- Use dedicated test-support modules when helpers are shared across tests.
- Do not automatically delete generated test artifacts unless the cleanup path
  is deliberately enabled and guarded.

## Routing Map

Use this map to choose the relevant detail file:

- General Rust implementation, constructors, file organization, async closures,
  iterator implementations, macro imports, and modern syntax:
  `rust-best-practices/general-best-practice.md`
- Cargo metadata, dependency grouping, resolver settings, crate edition, and
  lint defaults: `rust-best-practices/cargo-best-practices.md`
- Rust Edition 2024 features and migration-sensitive language behavior:
  `rust-best-practices/rust-edition-2024.md`
- Rust10x pack provenance and pack-level Rust 2024 guidance:
  `rust-best-practices/rust10x-pack.md`
- Error types, `Result` aliases, external error conversion, and test/example
  error handling: `rust-best-practices/error-best-practices.md`
- Unit tests, integration tests, test naming, test-support modules, temporary
  test data, and test result aliases:
  `rust-best-practices/test-best-practices.md`
- Comment regions and section markers:
  `rust-best-practices/comments-best-practices.md`
- Starting a new Rust crate or experiment:
  `rust-best-practices/new-project-best-practices.md`
- Rust example files under `examples/`:
  `rust-best-practices/examples-best-practices.md`
- CLI entry points, command parsing, executors, handlers, and clap boundaries:
  `rust-best-practices/cli-best-practices.md`
- Private module-level utilities and shared support code:
  `rust-best-practices/support-best-practices.md`
- Explicitly requested derive aliases:
  `rust-best-practices/derive-aliases.md`
- JSON-RPC or JOQL-style inter-process model APIs:
  `rust-best-practices/ipc-model-joql.md`
- Web API path conventions for JSON-RPC style APIs:
  `rust-best-practices/ipc-web-api.md`
- Pre-commit quality gates (fmt, clippy, test) and hook documentation:
  `rust-best-practices/quality-gates.md`

Apply CLI, example, derive-alias, JOQL, and Web API guidance only when work
actually touches those areas or an issue explicitly asks for that shape.
