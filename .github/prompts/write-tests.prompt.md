---
mode: ask
description: Generate unit tests and proptest property tests for a module
---

# Write Tests

Generate comprehensive tests for a module or function.

## Inputs

- **Target**: Module or function to test (e.g., `crates/parser.rs`)

## Steps

1. Read the target source file and identify all public functions
2. For each public function, create unit tests covering:
   - **Happy path**: Valid inputs produce expected outputs
   - **Error cases**: Invalid inputs return appropriate errors
   - **Edge cases**: Empty inputs, boundary values, special characters
3. Add property-based tests using `proptest` where applicable:
   - Roundtrip properties (encode/decode, serialize/deserialize)
   - Invariant properties (output always satisfies condition)
   - Commutativity or associativity where relevant
4. Place unit tests in `#[cfg(test)] mod tests` inside the source file
5. Place integration tests in `tests/` if they test cross-module behavior
6. Use descriptive names: `test_<function>_<scenario>_<expected>`
7. Run `cargo test --all-features` to verify all tests pass
