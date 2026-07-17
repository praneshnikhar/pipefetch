# Pipefetch — Plan

## Concept

A general-purpose HTTP CLI built for shell pipelines. Like `curl` meets `jq` meets Unix pipes — but designed from the ground up for composability. Every request is a **step** that yields structured data you can reference in the next step.

```
pipefetch get /users | pipefetch get "/users/{.id}/posts"
```

## Why not curl / httpie / hurl / getman?

| Tool | Pipeline story |
|------|---------------|
| `curl` | None. You wrap in shell functions. |
| `httpie` | None. Session files, but no piping. |
| `hurl` | Own file format. Can't compose in shell. |
| `getman` | TOML collections. No cross-request data flow. |
| **pipefetch** | First-class pipeline steps. Extract → inject in one command. |

## Architecture

```
pipefetch <step> [step ...]
```

Each step is one of:

| Step | Example |
|------|---------|
| `get <url>` | `pipefetch get https://api.example.com/users` |
| `post <url> <body>` | `pipefetch post /users '{"name":"Alice"}'` |
| `put <url> <body>` | `pipefetch put /users/1 '{"name":"Alice"}'` |
| `patch <url> <body>` | `pipefetch patch /users/1 '{"name":"Alice"}'` |
| `delete <url>` | `pipefetch delete /users/1` |
| `graphql <url> <query>` | `pipefetch graphql /graphql 'query { users { id } }'` |

### Output modes

| Flag | Effect |
|------|--------|
| (default) | Pretty-print response body (auto-color JSON, XML, HTML) |
| `--raw` | Raw response body, no formatting |
| `--status` | Print only the status line |
| `--headers` | Print only response headers |
| `--json` | JSON-only (machine-readable) |
| `--extract <jq-expr>` | Extract a specific value via JSONPath/jq-like syntax |

### Pipelining

The key innovation. Each step can reference values from previous steps:

```
pipefetch get /users --extract '[0].id' \
  pipefetch get "/users/{step[0]}/posts"
```

Or use named steps:

```
pipefetch --name users get /users \
  pipefetch --name posts get "/users/{users.id}/posts"
```

Reference syntax:

| Expression | What it resolves to |
|-----------|-------------------|
| `{step[N]}` | The response body of step N (auto-parsed if JSON) |
| `{step[N].field}` | JSON field from step N's response |
| `{name}` | Response body of named step |
| `{name.field}` | JSON field from named step |
| `{step[N].header.X}` | Response header from step N |
| `{step[N].status}` | Status code of step N |

### TOML collection mode

For complex workflows, define steps in a TOML file (like getman):

```toml
[client]
base = "https://api.example.com"
token = "${API_TOKEN}"

[step.users]
method = "GET"
path = "/users"
extract = "[0].id"

[step.posts]
method = "GET"
path = "/users/{users.value}/posts"
```

Run: `pipefetch run collection.toml`

## Key design decisions

1. **Shell-native, not file-native** — The primary use case is composing on the command line. TOML collections are for complex cases.
2. **Smart base URL** — If you pipe to pipefetch, the last URL is inherited. `pipefetch get /users | pipefetch get "{.id}"` inherits the base.
3. **Structured output by default** — Guess content-type, colorize JSON/XML, show response metadata.
4. **Idempotency-friendly** — `--dry-run`, `--quiet`, JSON report mode for CI.
5. **Auth profiles** — `pipefetch auth add github --type bearer --token ghp_xxx` — stored in `~/.config/pipefetch/profiles.toml`.

## Rust crate selection

| Need | Crate |
|------|-------|
| HTTP client | `reqwest` (with `rustls`-native-roots) |
| CLI framework | `clap` v4 (derive API) |
| JSON handling | `serde_json` + `serde` |
| JSONPath extraction | `serde_json_path` or custom jq-like mini-lang |
| TOML parsing | `toml` (or `toml_edit` for round-trip) |
| Color output | `owo-colors` or `colored` or `anstyle` |
| Table output | `tabled` |
| Async runtime | `tokio` |
| Config dirs | `dirs` |
| Templating | rolling our own simple `{ref}` resolver |
| Testing | `reqwest` mock server, `assert_cmd`, `predicates`, `insta` |

## Project structure

```
pipefetch/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, clap dispatch
│   ├── cli.rs               # CLI argument definitions
│   ├── client.rs            # HTTP client wrapper
│   ├── step.rs              # Step definition (URL, method, body, extract)
│   ├── pipeline.rs          # Pipeline execution: resolve refs → run steps
│   ├── resolver.rs          # {step[N].field} reference resolver
│   ├── output.rs            # Pretty-print, raw, JSON modes
│   ├── auth.rs              # Auth profiles management
│   ├── collection.rs        # TOML collection runner
│   └── config.rs            # Config file handling (~/.config/pipefetch/)
├── tests/
│   ├── integration.rs       # CLI tests with assert_cmd
│   └── pipeline_tests.rs    # Pipeline resolution tests
└── examples/
    └── demo.sh              # Example pipeline workflows
```

## Phased roadmap

### Phase 1 — MVP (Week 1)
- [x] Project scaffold (Cargo.toml, clap, reqwest setup)
- [x] Single-step HTTP methods: `get`, `post`, `put`, `patch`, `delete`
- [x] Pretty-print JSON responses
- [x] `--status`, `--headers`, `--raw` flags
- [x] Basic error handling (network errors, non-2xx)

### Phase 2 — Pipelines (Week 2)
- [x] `{step[N].field}` reference resolution
- [x] Named steps with `--name`
- [x] Multi-step pipeline execution
- [x] Shell pipe integration (detect piped input, inherit context)
- [x] `--extract` flag with JSONPath

### Phase 3 — Polish (Week 3)
- [x] Auth profiles (`pipefetch auth add/list/remove`)
- [x] TOML collection runner
- [x] `~/.config/pipefetch/` config
- [x] `--dry-run`, `--json` report mode
- [x] Environment variable interpolation (`${VAR}`)

### Phase 4 — Ship (Week 4)
- [x] Integration tests (assert_cmd + mock server)
- [x] CI/CD (GitHub Actions — test, lint, build)
- [x] Cross-platform releases (GitHub Releases with binaries)
- [x] Homebrew formula
- [x] Documentation (README, examples, man page)

### Post-v1
- [x] `graphql` step type
- [x] WebSocket support
- [x] gRPC support (via reflection)
- [x] Shell completions (bash, zsh, fish)
- [x] Vim/Helix plugin for TOML collection files
- [x] `pipefetch watch` — poll an endpoint periodically

## Testing strategy

| Layer | Tool | What |
|-------|------|------|
| Unit | `cargo test` | Resolver, step parser, output formatting |
| Integration | `assert_cmd` + `predicates` | CLI as a black box |
| HTTP mocking | `wiremock` or `httpmock` | Test against fake servers |
| Snapshot | `insta` | Golden files for output formats |
| Pipeline | Custom harness | Chain steps, verify resolved URLs |

## Release & distribution

- GitHub Actions — `cargo test` on push, `cargo build --release` on tag
- GitHub Releases with binaries via `softprops/action-gh-release`
- Homebrew tap: `brew install pipefetch/tap/pipefetch`
- Cargo install: `cargo install pipefetch`
- Manual: download from GitHub Releases

## Naming

Suggestions (need to decide):
- **`pipefetch`** — descriptive, implies piped fetching
- **`pcurl`** — pipe + curl, familiar
- **`httppipe`** — literal
- **`flow`** — short, evocative

## Questions to resolve

1. Naming
2. Extractor DSL: JSONPath, jq-style, or something simpler?
3. TOML vs YAML for collections?
4. Async reqwest vs blocking for MVP?
