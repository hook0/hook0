# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0-alpha.3] - 2026-02-19

### CI

- Fix GitLab releases
- Fix output worker package name

## [1.0.0-alpha.2] - 2026-02-13

### Added

- Add Pulsar metrics (output-worker)

### CI

- Improve release content
- Improve release content
- Add frontend container build and GitLab release to pipeline
- Update tools
- Improve GitLab releases contents

### Fixed

- Switch release containers from docker:dind to BuildKit rootless (ci)
- Escape quotes in BuildKit --output for CSV parser (ci)

### Other

- Update dependencies

## [1.0.0-alpha.1] - 2026-02-09

### Fixed

- Switch release containers from docker:dind to BuildKit rootless (ci)
- Escape quotes in BuildKit --output for CSV parser (ci)
