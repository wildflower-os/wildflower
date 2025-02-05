# Changelog
All notable changes to Wildflower after the MOROS fork will be documented in this file.

Please note, this does not include minor errors.

## [Unreleased]

## [0.1.0-alpha3] - 2025-02-05
- Add persistence to the HFS
- Add bounds checking to the HFS
- Use `nasm` for rebooting (Currently breaks the `print/echo` and the `sleep` commands, as the `nasm` counterparts are not implemented yet)

## [0.1.0-alpha2] - 2025-02-03
- Removal of multiple TODOs
- Removal of multiple FIXMEs
- More test cases
- Centralize games under the `play` command
- Update `nom` from 7.1.3 to 8.0.0

## [0.1.0-alpha1] - 2025-01-29
- Revert the end-user friendly changes, as this caused memory allocation issues

## [0.1.0-alpha0] - 2025-01-28
- Initial Fork
- Dependency Updates
- Wildflower Branding
- HFS (Hidden File System)
- Easier Makefile commands
- More end-user friendly
