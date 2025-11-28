# Development Guidelines

## Test-Driven Development (TDD)

This project follows a Test-Driven Development approach. All new features and functionalities must include unit tests.

### Requirements:

1. **All modules must have unit tests**
   - Write tests before implementing functionality
   - Each public function should have corresponding test cases
   - Test both success and failure paths

2. **Module Structure**
   - Keep main.rs minimal - it should only handle CLI parsing and calling modules
   - Implement functionality in separate modules under `src/`
   - Each module should be in its own file or directory

3. **Test Organization**
   - Unit tests should be in the same file as the code (using `#[cfg(test)]` modules)
   - Integration tests go in the `tests/` directory
   - Test modules should be named `tests` or `test_[feature_name]`

4. **Running Tests**
   ```bash
   cargo test              # Run all tests
   cargo test --lib        # Run only unit tests
   cargo test module_name  # Run tests for specific module
   ```

5. **Code Coverage**
   - Aim for high test coverage
   - Critical paths (auth, token storage) must be thoroughly tested

## Module Structure

The project is organized as follows:

```
src/
├── main.rs           # CLI entry point only
├── auth/             # Authentication module
│   ├── mod.rs        # Module exports
│   ├── oauth.rs      # OAuth flow implementation
│   └── tokens.rs     # Token storage
├── config/           # Configuration management
└── calendar/         # Google Calendar API integration
```

## Future Development

When adding new features:
1. Create/update module in `src/`
2. Write tests first (TDD)
3. Implement functionality to pass tests
4. Ensure all tests pass before committing
