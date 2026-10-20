# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |

## Reporting a vulnerability

**Please do not open a public issue for security problems.**

Email `security@artcraft.dev` with:
- A description of the issue
- Steps to reproduce
- The affected version(s)

We will acknowledge within 72 hours and aim to ship a fix within 14 days for critical issues.

## Scope

Renosancse Craft spawns `ffmpeg`/`ffprobe` as subprocesses with paths controlled by the user's file picker. We do not implement network listeners. The primary attack surface is:
- Malicious audio files triggering FFmpeg vulnerabilities (mitigated by upstream FFmpeg updates — always ship the latest)
- Path handling in the file queue

Out of scope: FFmpeg's own vulnerabilities (report upstream), OS-level sandbox escapes.
