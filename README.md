# PolyCOM

A simple and stupid LAN VoIP application.

---

## Requirements

You only need:

- Git
- Internet connection

Everything else (Rust, toolchain, components) can be installed from this repo.

---

## 🦀 Rust Toolchain

This project uses **rustup** (official Rust installer and version manager):
https://rustup.rs

The required toolchain is defined in `rust-toolchain.toml`.

It ensures everyone uses the same Rust version and components.

---

## 📦 Project Setup

### Windows (PowerShell)

Run:

```powershell
.\scripts\setup.ps1
```

### macOS / Linux

Run:

``` bash
chmod +x scripts/setup.sh
./scripts/setup.sh
```

### ⚙️ What the setup scripts do

They automatically:

- Install Rust (if missing)
- Install required components:
- rustfmt (formatter)
- clippy (linter)
- Ensure toolchain matches project definition

## Running the project

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

### Test

```bash
cargo test
```

## Code formatting
This project uses `rustfmt`, the official Rust formatter.

Format code:
```bash
cargo fmt
```

Check formatting in CI-style mode:

```bash
cargo fmt --all -- --check
```
## Linting

We use:
- clippy (official Rust linter)

Run:

```bash
cargo clippy
```

Treat warnings as errors (CI mode):

```bash
cargo clippy -- -D warnings
```

## Continuous Integration

CI runs automatically on:
- push to `develop`
- pull requests on `develop`

It checks:
- Formatting (rustfmt)
- Linting (clippy)
- Build
- Tests

## Recommended VS Code setup
Install suggested extensions when prompted:

Extensions:
- rust-analyzer
- CodeLLDB
- Even Better TOML
- dependi

## Dependency Management

Dependencies are defined in `Cargo.toml`.

Lockfile: `Cargo.lock`

## Useful commands
```bash
cargo fmt        # format code
cargo clippy     # lint code
cargo build      # build project
cargo run        # run project
cargo test       # run tests
```

## Troubleshooting

### Rust not found
Run:
```bash
curl https://sh.rustup.rs -sSf | sh
```
or on Windows:
Run `scripts/setup.ps1`.

### Toolchain mismatch

Run:

```bash
rustup show
rustup update
```

## 🤝 Contributing
- [ ] Use feature branches
- [ ] Ensure `cargo fmt` passes
- [ ] Ensure `cargo clippy` passes
- [ ] Ensure tests pass before PR
