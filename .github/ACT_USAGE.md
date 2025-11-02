# Running GitHub Actions Locally with Act

This guide explains how to test GitHub Actions workflows locally using `act`.

## Prerequisites

1. **Docker** - Must be installed and running
   - Verify: `docker --version`
   - Download: https://www.docker.com/get-started

2. **Act CLI** - Tool to run GitHub Actions locally
   - Windows: `winget install nektos.act`
   - macOS: `brew install act`
   - Linux: Use the install script from the [act repository](https://github.com/nektos/act)

## Basic Usage

### List all workflows
```bash
act -l
```

### Run a specific workflow event
```bash
# Run on push event
act push

# Run on pull_request event
act pull_request
```

### Run a specific job
```bash
# List jobs first
act -l

# Run a specific job (e.g., the CI job)
act -j ci
```

### Dry run (see what would happen)
```bash
act --dryrun
```

## Common Options

### Specify workflow file
```bash
act -W .github/workflows/ci.yml
```

### Use a specific event
```bash
act push -e .github/push-event.json
```

### Skip jobs
```bash
act -j lint-frontend --skip test-frontend
```

### Verbose output
```bash
act -v
```

### Use different runner image
```bash
act -P ubuntu-latest=catthehacker/ubuntu:act-latest
```

## Important Notes

1. **Docker must be running** - Act uses Docker containers to simulate GitHub Actions runners
2. **Some actions may not work locally** - Actions that require GitHub-specific features (like artifact uploads) may fail
3. **Secrets** - If workflows need secrets, use a `.secrets` file:
   ```bash
   # .secrets
   GITHUB_TOKEN=your-token-here
   ```
   Then run: `act --secret-file .secrets`

4. **Matrix strategies** - Act will run all matrix combinations by default

5. **Performance** - Local runs may be slower than GitHub Actions due to Docker overhead

## Example: Testing CI Workflow

```bash
# Run the full CI workflow
act push -W .github/workflows/ci.yml

# Run only the lint job
act push -j lint-frontend -W .github/workflows/lint.yml

# Run with verbose output to debug issues
act push -v -W .github/workflows/test.yml
```

## Troubleshooting

- **"Cannot connect to Docker daemon"** - Make sure Docker Desktop is running
- **Actions fail with permission errors** - Try running with `--container-options "-u root"`
- **Cache not working** - Local caching behavior may differ from GitHub Actions

## Resources

- [Act GitHub Repository](https://github.com/nektos/act)
- [Act Documentation](https://github.com/nektos/act#readme)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

