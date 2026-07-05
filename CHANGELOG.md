# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and the project follows Semantic Versioning.

## [Unreleased]

- Initial open source project scaffolding

## [0.1.1] - 2026-07-05

- Fixed `needled` readiness semantics so queries fail until the initial index is ready, and taught `ndl` to wait for daemon readiness before issuing search requests
- Moved daemon reindex work out of the global state mutex to avoid blocking all requests during full rebuilds
- Fixed Linux inotify path resolution to use watch descriptors directly, which corrects delete handling and duplicate-basename collisions across watched directories
- Added watcher health reporting to daemon status, including watch coverage, watch failures, and inotify overflow counts
- Triggered full daemon reindex after inotify overflow so stale watcher state is repaired instead of silently persisting
- Reduced watcher startup lock contention and expanded tests around daemon readiness and wd-based watcher resolution

## [0.1.0] - 2026-07-03

- Initial public workspace structure for `needle-core`, `needled`, and `ndl`
