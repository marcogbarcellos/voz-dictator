# Contributing to Voz

Thanks for your interest in contributing to Voz! Here's how to get started.

## Development Setup

See the [README](README.md#local-setup) for full setup instructions.

Quick start:

```bash
git clone https://github.com/marcogbarcellos/voz.git
cd voz
pnpm install
cargo install tauri-cli --version "^2"
cd apps/desktop
cargo tauri dev
```

## Making Changes

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Test locally with `cargo tauri dev`
4. Submit a pull request

## Code Style

- **Rust**: Follow standard `rustfmt` formatting. Run `cargo fmt` before committing.
- **TypeScript/React**: Use the existing Tailwind CSS patterns. No additional linter is configured yet — just match the existing style.

## Pull Requests

- Keep PRs focused on a single change
- Describe what you changed and why
- Include screenshots for UI changes

## Reporting Bugs

Open a GitHub issue with:
- Steps to reproduce
- Expected vs actual behavior
- macOS version and system info

## Feature Requests

Open a GitHub issue describing the feature and why it would be useful.

## Security Vulnerabilities

Please report security issues privately — see [SECURITY.md](SECURITY.md).
