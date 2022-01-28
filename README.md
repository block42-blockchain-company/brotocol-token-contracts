[![Tests and Linter checks](https://github.com/block42-blockchain-company/brotocol-token-contracts/actions/workflows/basic.yml/badge.svg)](https://github.com/block42-blockchain-company/brotocol-token-contracts/actions/workflows/basic.yml)
[![Security audit](https://github.com/block42-blockchain-company/brotocol-token-contracts/actions/workflows/audit.yml/badge.svg)](https://github.com/block42-blockchain-company/brotocol-token-contracts/actions/workflows/audit.yml)

---
# Brotocol Token Contracts

## Pipelines
- basic.yml - runs unit tests(cargo test) and linter commands (cargo fmt, cargo clippy)
- codecov.yml - runs cargo tool which checks current code coverage via implemented unit tests
- audit.yml - audit Cargo.lock files for crates with security vulnerabilities reported to the RustSec Advisory Database.

TBA