# git-identity

A CLI tool for managing git identity profiles. Quickly switch between different git identities (name, email, signing key) on a per-repository basis.

## Installation

```bash
cargo install --path .
```

## Usage

### List available identities

```bash
git-identity list
```

Output:
```
Available git identities:
- personal
* work
```

The `*` indicates the currently active identity for the repository.

Use `-v` for verbose output with full details:

```bash
git-identity list -v
```

### Set an identity

```bash
git-identity set work
```

This applies the identity to the current repository's local git config.

## Configuration

Create a profiles file at `~/.git-identities/profiles`:

```ini
[identity "personal"]
    name = Your Name
    email = you@personal.com

[identity "work"]
    name = Your Name
    email = you@company.com
    signingkey = ABC123
```

### Profile options

| Option       | Description                          | Required |
|--------------|--------------------------------------|----------|
| `name`       | The git `user.name` value            | Yes      |
| `email`      | The git `user.email` value           | Yes      |
| `signingkey` | The git `user.signingkey` for commits| No       |

## How it works

- `git-identity list` reads profiles from `~/.git-identities/profiles` and displays them. It also checks the current repository's git config to show which identity is active.

- `git-identity set <name>` finds the matching profile and writes `user.name`, `user.email`, and optionally `user.signingkey` to the current repository's local `.git/config`.

## Contributing

Contributions are welcome! The codebase is documented with rustdoc comments. Run `cargo doc --open` to browse the documentation.

### Project structure

```
src/
  main.rs      # CLI entry point and argument parsing
  identity.rs  # Core identity management logic
  error.rs     # Error types
```

## License

MIT