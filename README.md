# pipefetch

HTTP client for shell pipelines — pipe JSON responses directly into `jq`, `yq`, or any other tool.

## Usage

```bash
pipefetch get https://api.example.com/users
pipefetch post /users '{"name":"Alice"}' --status
pipefetch get https://api.example.com/users | jq '.[].name'
```

## Install

```bash
cargo install --path .
```
