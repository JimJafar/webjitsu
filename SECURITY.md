# Security Policy

WebJitsu is a [fork of Bladebro](https://github.com/dondai44423/bladebro); see
[FORK.md](FORK.md) for what this build changes. Report a vulnerability in
upstream's code through upstream's policy below; report a vulnerability in the
fork's gates as a [GitHub issue](https://github.com/JimJafar/webjitsu/issues)
with the `security` label, or privately by email (below) if you prefer.

## Reporting a vulnerability

If you discover a security vulnerability in WebJitsu, please report it privately:

1. **Do not open a public issue.**
2. Email the maintainer directly: **bhandaribishesh879@gmail.com**
3. Include a description of the vulnerability, steps to reproduce, and potential impact.

You will receive a response within 48 hours. If the vulnerability is confirmed, a fix will be prioritized and a security advisory published via GitHub Security Advisories.

## Scope

- Vulnerabilities in WebJitsu's code (Rust source)
- Stealth bypasses that would expose agent sessions to detection
- Path traversal or injection in session save/load
- CDP connection handling (e.g., Chrome death, resource exhaustion)

## Out of scope

- Using WebJitsu to bypass website terms of service
- Bot detection on sites that require CAPTCHA solving (WebJitsu detects and reports honestly — it does not solve as a general capability; the one exception is adapter-specific and human-trivial, e.g. reddit's one-time humanity checkbox)
- Vulnerabilities in Chromium itself (report to the Chromium project)
