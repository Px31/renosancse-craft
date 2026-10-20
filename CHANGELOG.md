# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Magic-byte format detection for MP3, M4A, WAV, FLAC, OGG, Opus, AAC, AIFF, WMA, AC-3
- `ffprobe` metadata probing with graceful fallback
- Recommendation engine: lossless → FLAC, lossy → MP3 320 kbps, default → WAV
- FFmpeg command builder with codec routing and metadata preservation
- iced-based GUI shell with batch queue
- Cross-platform CI (Windows / Debian / Fedora) and release workflow

## [0.1.0] - 2025-01-01

Initial skeleton release.
