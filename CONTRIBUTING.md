# Contributing to Renosancse Craft

Thanks for considering a contribution. Here's how to get set up and what we expect.

## Build prerequisites

### Windows
- Rust via [rustup](https://rustup.rs)
- FFmpeg on `PATH` (`winget install Gyan.FFmpeg`)
- WebView not required — `iced` uses `wgpu` natively.

### Debian / Ubuntu
```bash
sudo apt install -y build-essential pkg-config libasound2-dev \
  libxkbcommon-dev libwayland-dev ffmpeg
```

### Fedora
```bash
sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config \
  alsa-lib-devel libxkbcommon-devel wayland-devel ffmpeg
```

## Running

```bash
cargo run --release
```

## Before you open a PR

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

CI runs all three on every PR. If CI is green, you're good.

## What we're looking for

- **Small, focused PRs.** One feature or fix per PR. We'd rather review five small PRs than one giant one.
- **Tests for logic.** Detection, recommendation, and command-building functions should have unit tests. GUI code is exempt (though snapshot tests are welcome).
- **No new heavy dependencies** without discussion first. Open an issue tagged `dependency`.
- **Keep FFmpeg optional.** New code must not require FFmpeg to compile. The `Converter` trait is the seam—extend it, don't bypass it.

## Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). Be kind.
