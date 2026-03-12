---
name: write-tests
description: Generate tests for a function, module, or file. Use when the user asks to add or write tests.
---

Write tests for the specified code. Before writing:
1. Read the target code and any existing test files to match the project's test framework, style, and conventions
2. Identify the test framework in use (check package.json, pyproject.toml, Cargo.toml, etc.)

Cover: happy path, edge cases, error conditions. Keep tests focused — one assertion per behavior. Use descriptive test names that explain the expected behavior. Don't mock what you don't need to.
