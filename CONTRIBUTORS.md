# Contributing Guide

Thank you for your interest in contributing to the iTunes to MTP Sync project! This guide will help you understand our development workflow and coding standards.

## Git Flow Workflow

This project uses **Git Flow** for managing branches and releases. Please follow this workflow when contributing.

### Branch Structure

```
main (production)
  └── development (integration)
       └── feature/* (feature branches)
```

### Branch Types

#### **main** (Production Branch)
- **Protected**: Yes - Direct commits are **NOT allowed**
- **Purpose**: Contains production-ready code
- **Updates**: Only via merge from `development` branch
- **Tagging**: Tagged with version numbers for releases

#### **development** (Integration Branch)
- **Protected**: Yes - Direct commits are **NOT allowed**
- **Purpose**: Integration branch for new features
- **Updates**: Only via merge from `feature/*` branches
- **Testing**: Should always be in a deployable state

#### **feature/\*** (Feature Branches)
- **Protected**: No
- **Purpose**: Development of new features or bug fixes
- **Naming**: Must start with `feature/` (e.g., `feature/playlist-parsing`, `feature/fix-mtp-upload`)
- **Updates**: Created from and merged into `development`

### Workflow Steps

#### 1. Create a Feature Branch

Always start from the latest `development` branch:

```bash
# Update your local branches
git fetch origin

# Checkout development
git checkout development

# Pull latest changes
git pull origin development

# Create a new feature branch
git checkout -b feature/your-feature-name
```

#### 2. Make Changes

- Write code following our coding standards
- Write tests for new features (80% coverage minimum)
- Ensure all linters pass
- Commit frequently with meaningful commit messages

#### 3. Commit Message Format

We use **Conventional Commits** format. All commit messages must follow this structure:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, whitespace, etc.)
- `refactor`: Code refactoring (no functional changes)
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `build`: Build system or dependency changes
- `ci`: CI/CD configuration changes
- `chore`: Other changes (maintenance tasks)

**Examples:**
```
feat(mtp): add file upload functionality

fix(parser): handle missing track metadata

docs(readme): update installation instructions

test(sync): add unit tests for sync service
```

**Note**: Husky hooks will automatically validate commit messages. Invalid formats will be rejected.

#### 4. Push and Create Merge Request

```bash
# Push your feature branch
git push origin feature/your-feature-name
```

Then create a merge request to `development` branch.

#### 5. Code Review

- All code must be reviewed before merging
- Address review comments
- Update your branch if requested

#### 6. Merge to Development

Once approved, your feature branch will be merged into `development`.

#### 7. Merge Development to Main

After features in `development` are tested and ready for release, `development` is merged into `main`.

**Important**: Only maintainers can merge `development` → `main`.

### Branch Protection Rules

#### Protected Branches: `main` and `development`

**Rules enforced by Husky:**
1. ❌ **Direct commits are blocked** - You cannot commit directly to `main` or `development`
2. ✅ **Feature branch merges allowed** - Feature branches can merge into `development`
3. ✅ **Development → Main allowed** - Only `development` can merge into `main`

#### Workflow Enforcement

Husky pre-push hooks will automatically:
- Block direct commits to `main` (except merges from `development`)
- Block direct commits to `development` (except merges from `feature/*` branches)
- Allow feature branches to merge into `development`
- Allow `development` to merge into `main`

### Pre-commit Hooks

Before every commit, the following checks run automatically:

1. **Linting** (`pnpm lint`)
   - ESLint for TypeScript/Angular code
   - Must pass before commit

2. **Testing** (`pnpm test`)
   - Vitest tests must pass
   - All tests must be green

3. **Rust Linting** (`cargo clippy`)
   - Rust code must pass clippy checks
   - Warnings are treated as errors

**Note**: If any check fails, the commit will be blocked. Fix issues and try again.

### Commit Message Validation

The commit-msg hook validates commit messages:
- Must follow Conventional Commits format
- Type must be one of: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
- Subject cannot be empty
- Header must be 100 characters or less

### Getting Help

If you encounter issues:

1. **Git Flow questions**: Check this guide or open an issue
2. **Husky hook failures**: Check the error message and fix the issues
3. **Commit message rejected**: Review the Conventional Commits format
4. **Branch protection errors**: Ensure you're following the workflow correctly

### Quick Reference

```bash
# Start new feature
git checkout development
git pull origin development
git checkout -b feature/my-feature

# Make changes and commit (with proper message)
git add .
git commit -m "feat(scope): add new feature"

# Push and create merge request
git push origin feature/my-feature

# After merge, clean up
git checkout development
git pull origin development
git branch -d feature/my-feature
```

### Emergency Hotfixes

For critical production bugs, hotfix branches can be created from `main`:

```bash
git checkout main
git pull origin main
git checkout -b hotfix/critical-bug-fix

# Make fix, commit, push
# Merge hotfix to both main and development
```

## Development Setup

### Prerequisites

- Node.js (see `.nvmrc` or `package.json` engines)
- pnpm (package manager)
- Rust toolchain (for Tauri backend)
- Git

### Setup

```bash
# Clone repository
git clone <repository-url>
cd itunes-mtp

# Install dependencies
pnpm install

# Install Husky hooks (runs automatically via prepare script)
# Hooks are installed when you run pnpm install

# Verify setup
pnpm lint
pnpm test
```

## Code Standards

- **TypeScript/JavaScript**: Follow `.cursor/rules/angular-typescript.mdc`
- **Rust**: Follow `.cursor/rules/rust.mdc`
- **Testing**: Follow `.cursor/rules/testing.mdc`
- **JavaScript Files**: Use `.mjs` extension only (ESM mode)
- **Linting**: All code must pass linting
- **Tests**: 80% minimum coverage required

## Questions?

If you have questions about the workflow or encounter issues:
1. Check this guide
2. Review existing issues
3. Create a new issue with the `question` label

Thank you for contributing! 🎉

