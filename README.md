# pipefetch

HTTP client for shell pipelines.

```bash
# Fetch and extract
pipefetch get https://api.github.com/repos/praneshnikhar/pipefetch --extract .description

# Chain requests — pipe extracted values into templates
pipefetch get /users --extract '[0].id' | pipefetch get "/users/{.}"

# Run multi-step YAML collections
pipefetch run examples/demo.yaml
```

## Install

### From source
```bash
cargo install --git https://github.com/praneshnikhar/pipefetch
```

### Binary downloads
Download the latest release from [GitHub Releases](https://github.com/praneshnikhar/pipefetch/releases).

## Commands

### HTTP methods
```bash
pipefetch get <url>
pipefetch post <url> <body>      # body is JSON (auto-sets Content-Type)
pipefetch put <url> <body>
pipefetch patch <url> <body>
pipefetch delete <url>
```

### Output modes
| Flag | Description |
|------|-------------|
| *(default)* | Status line, headers, pretty-printed JSON body |
| `--status` | Print only the status line (e.g. `200 OK`) |
| `--headers` | Print only response headers |
| `--raw` | Print raw response body, no formatting |
| `--json` | Machine-readable JSON output — `{status, headers, body}` |

### Extraction & pipelines
| Flag | Description |
|------|-------------|
| `--extract .path` | Extract a JSON value via dot notation |

Dot notation supports:
- `.field` — object key
- `.field.subfield` — nested access
- `[0]` — array index
- `.field[0].subfield` — mixed

```bash
# Extract a string value
pipefetch get /user --extract .name           # → Alice

# Pipe extracted value → next request
pipefetch get /repo --extract .owner.login |
  pipefetch get "/users/{.}"
```

Template resolution `{.path}` reads from piped context:
- `{.}` — entire piped value
- `{.field}` — field from piped JSON
- `{.field[0]}` — array element

### Environment variables
Use `${VAR}` in URLs and bodies — resolved from process environment:

```bash
pipefetch get "https://api.github.com/repos/${OWNER}/${REPO}" --extract .description
```

Works alongside pipe templates — env vars resolve first, then pipe context.

### Auth profiles
```bash
pipefetch auth add prod --auth-type bearer --value ghp_xxx
pipefetch auth add staging --auth-type basic --value admin:secret
pipefetch auth list
pipefetch auth remove prod
```

Use profiles when making requests:
```bash
pipefetch get /users --auth prod
```

Profiles are stored at `~/.config/pipefetch/config.yaml` (Linux) or `~/Library/Application Support/pipefetch/config.yaml` (macOS).

### Config file
Create `config.yaml` for defaults:
```yaml
default_base: https://api.example.com
auth:
  - name: prod
    type: bearer
    value: ghp_xxx
```

With `default_base` set, relative URLs resolve automatically:
```bash
pipefetch get /users    # → GET https://api.example.com/users
```

### Dry run
Preview a request without sending it:
```bash
pipefetch get /users --auth prod --dry-run
# GET /users
# Authorization: Bearer ghp_xxx
```

### Collections
Run multi-step YAML collections where steps can reference each other:

```yaml
# examples/demo.yaml
steps:
  - name: get_uuid
    method: GET
    path: https://httpbin.org/uuid
    extract: .uuid

  - name: verify
    method: GET
    path: https://httpbin.org/anything/{get_uuid}
    status: 200
    extract: .url
```

```bash
pipefetch run examples/demo.yaml
# [OK] 200 get_uuid → 7c06ad58-...
# [OK] 200 verify   → https://httpbin.org/anything/7c06ad58-...
```

Collection features:
- `{step_name}` references the extracted value of a prior step
- `status:` asserts expected status code (exits 1 on mismatch)
- `body:` accepts YAML (auto-converted to JSON) or raw strings
- `headers:` per-step and client-level
- `client.base:` base URL for relative paths
- `client.auth:` auth profile name

## Examples

```bash
# Simple GET
pipefetch get https://httpbin.org/get

# POST with JSON body
pipefetch post https://httpbin.org/post '{"hello":"world"}'

# Extract and chain
pipefetch get /repos/praneshnikhar/pipefetch --extract .owner.login |
  pipefetch get "/users/{.}" --extract .name

# Check status in CI
pipefetch get https://api.example.com/health --status

# JSON report for scripting
pipefetch get /users --json | jq '.body | length'

# Abbreviated response
pipefetch get /users --extract '[0].name'
```

## Development

```bash
cargo test
cargo clippy
cargo fmt
```

## License

GPL-3.0
