# CI/CD Workflows

This directory contains GitHub Actions workflows for automated testing, linting, and quality assurance.

## Workflows

### `ci.yml`
Main CI pipeline that runs all checks in a single workflow. This is a quick validation that runs on every push and pull request.

### `test.yml`
Comprehensive testing workflow that:
- Runs frontend tests (Vitest) on multiple Node.js versions and platforms
- Runs backend tests (Rust) on multiple Rust versions and platforms
- Generates and uploads coverage reports
- Runs integration tests

### `lint.yml`
Linting workflow that:
- Runs ESLint for TypeScript/Angular code
- Runs Rust clippy for Rust code
- Checks code formatting
- Validates commit messages (if commitlint is configured)

## Requirements

### Frontend Testing
- Node.js 20.x or 22.x
- pnpm package manager
- Vitest test framework

### Backend Testing
- Rust stable or 1.75.0
- cargo-tarpaulin (for coverage on Linux)

## Coverage Reports

Coverage reports are generated and uploaded as artifacts:
- Frontend: Uploaded for each platform/Node version combination
- Backend: Uploaded for Linux platform only (tarpaulin limitation)

Reports are available in the workflow artifacts for 7 days after run completion.

## Running Locally

To run the same checks locally:

```bash
# Install dependencies
pnpm install

# Run linting
pnpm lint

# Run frontend tests
pnpm test

# Run frontend tests with coverage
pnpm test:coverage

# Run Rust tests
cd src-tauri
cargo test

# Run Rust clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check Rust formatting
cargo fmt --all --check
```

## Future Work

- Release pipeline (to be created separately)
- Automated deployment workflows
- Docker image builds
- Performance benchmarking

