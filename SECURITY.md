# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Voz, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email: **marcogbarcellos@gmail.com**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact

You should receive a response within 72 hours. We will work with you to understand and address the issue before any public disclosure.

## Scope

Security issues we care about:
- Credential leakage (API keys stored insecurely beyond the documented `settings.json`)
- Unauthorized access to microphone data
- Code injection via transcription or text injection pipeline
- Clipboard data exposure

## API Key Storage

Voz stores API keys in plaintext at `~/.config/voz/settings.json`. This is documented and by design — users should treat this file as sensitive and set appropriate file permissions.
