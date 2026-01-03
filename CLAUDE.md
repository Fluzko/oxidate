# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A TUI calendar application with Google Calendar integration. Currently implements OAuth2 authentication flow with Google Calendar API. Future development will add calendar viewing and management features through a terminal user interface.

## Development Commands

**Build the project:**
```bash
cargo build
```

**Run the project:**
```bash
cargo run
```

**Run tests:**
```bash
cargo test
```

**Check code without building:**
```bash
cargo check
```

**Run clippy linter:**
```bash
cargo clippy
```

**Format code:**
```bash
cargo fmt
```

## Project Configuration

- **Edition**: Rust 2021
- **Version**: 0.1.0
- **Main Dependencies**: oauth2, reqwest, tokio, serde, clap, google-calendar3, anyhow
- **Credentials**: OAuth2 credentials embedded at compile-time via build.rs (development uses .env file)
- **Token Storage**: `~/Library/Application Support/oxidate/token.json` (macOS) or `~/.config/oxidate/token.json` (Linux)

## Architecture & Development Patterns

### OOP in Rust

**Pattern: Structs + impl blocks for entity-based organization**

Organize functionality around structs that represent entities (Tokens, OAuthClient, PortSelector). Each struct owns its data and provides methods to operate on that data.

**Method Types:**
- **Associated functions**: Operations not tied to specific instance (load, exists, delete) - like static methods
- **Instance methods**: Operations on specific data (save) - operate on self
- **Private helpers**: Shared logic as associated functions when not instance-specific

**Benefits:** Encapsulation, intuitive API, easy testing per method

### TDD Workflow

**Steps:**
1. Write test with expected behavior
2. Implement minimal code to pass
3. Refactor
4. Repeat

**Test Structure:**
- Cleanup/Setup: Prepare clean state
- Execute: Run the operation being tested
- Assert: Verify expected behavior
- Cleanup: Remove test artifacts

**Coverage Strategy:**
- Test each public method
- Test success path + error cases
- Test edge cases (file doesn't exist, invalid data)
- Use descriptive names: `test_load_fails_when_file_missing` not `test1`

**Key Insight:** Tests catch type mismatches early, serve as documentation, give confidence for refactoring

### Modular Architecture

**Structure:**
- `main.rs`: Minimal orchestration only (CLI parse → workflow → display)
- `cli.rs`: CLI argument parsing
- `auth/mod.rs`: Public API + high-level authenticate() workflow
- `auth/tokens.rs`: Token persistence
- `auth/port.rs`: Port selection
- `auth/oauth.rs`: OAuth flow

**Rules:**
- One responsibility per module
- Main.rs = orchestration only, no business logic
- Module mod.rs exports public API, internal details stay private
- Loose coupling between modules

**Benefits:** Changes isolated, easy to find code, modules reusable, easy to add new features

### Error Handling with anyhow

**Approach:**
- Use `anyhow::Result<T>` as return type (not custom error types)
- Propagate errors with `?` operator
- Add context to errors with `.context("message")` for debugging
- Use `anyhow::bail!("msg")` for early returns
- Convert Options to Results with `.context("reason")`

**Error Flow:**
Errors propagate up the call stack, accumulating context at each layer. Main.rs handles final error display and exits with appropriate code.

**Benefits:** No custom error types needed, context chains up, compiler enforces handling, works with any error type

### Commit Strategy

**When to Commit:**
Commit after each **complete logical unit**. A logical unit is complete when:
1. The feature/method/function is fully implemented
2. All related tests pass
3. The code compiles without errors

**Examples of Logical Units:**
- A constructor method (`new()`) with its tests
- A single public method with its implementation and tests
- A bug fix that makes previously failing tests pass
- A refactoring that maintains all passing tests

**Workflow - ALWAYS Follow This Sequence:**
1. **Implement** the feature/method
2. **Run tests** for that specific unit: `cargo test module::tests::test_name`
3. **Verify** all tests pass
4. **Commit** with descriptive message
5. **Push** to remote
6. **Move to next** logical unit

**Commit Message Format:**
```
[TICKET-ID] Brief description of what was implemented

Detailed explanation of changes:
- What was added/changed
- Why it was necessary
- Any important implementation details
```

**Decision Criteria - When to Commit:**
- ✅ **DO commit**: After implementing `CalendarClient::new()` and its tests pass
- ✅ **DO commit**: After fixing a bug and the specific test now passes
- ✅ **DO commit**: After implementing `list_calendars()` method completely
- ❌ **DON'T commit**: After writing only the function signature with `todo!()`
- ❌ **DON'T commit**: With failing tests
- ❌ **DON'T commit**: Multiple unrelated changes together

**Important Rules:**
- NEVER commit without running tests first
- NEVER commit failing tests
- NEVER batch multiple logical units into one commit
- ALWAYS push immediately after committing
- ALWAYS run `cargo test` before commit, not just the specific test
- NEVER add references to Claude Code, Anthropic, or AI tools in commit messages or PR descriptions

**Branch Naming:**
- Format: `TICKET-ID` (e.g., `MFLP-8`)
- Create branch at start of ticket work
- One branch per ticket

## Quick Reference for Adding Features

**Steps:**
1. Create module in `src/feature_name/`
2. Define structs with OOP methods
3. Write tests first (TDD)
4. Implement with `anyhow::Result`
5. Export public API in mod.rs
6. Call from main.rs

**Testing checklist:**
- [ ] Success path
- [ ] Error cases
- [ ] Edge cases
- [ ] Cleanup after tests
- [ ] Descriptive names

## Jira Workflow

**Status Columns:**
The project uses these exact column names in Jira:
- `POR HACER` - To Do
- `EN CURSO` - In Progress
- `EN REVISIÓN` - In Review
- `FINALIZADO` - Done

**Important:** Use ONLY these exact column names. Never attempt to use different names.

**Workflow:**
1. Start ticket → Move to `EN CURSO`
2. Create branch: `TICKET-ID` format
3. Implement feature following TDD + Commit Strategy
4. Create PR → Move ticket to `EN REVISIÓN`
5. After PR merge → Move ticket to `FINALIZADO`

**When to Move Tickets:**
- Always ask before moving a ticket to `FINALIZADO`
- Automatically move to `EN REVISIÓN` when PR is created