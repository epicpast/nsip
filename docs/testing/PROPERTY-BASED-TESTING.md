---
diataxis_type: reference
---
# Property-Based Testing Guide

## Overview

Property-based testing validates code properties hold for all inputs, not just specific examples. Uses [proptest](https://github.com/proptest-rs/proptest) and [quickcheck](https://github.com/BurntSushi/quickcheck).

**Philosophy:** Instead of testing specific cases, test universal properties.

## Traditional vs Property-Based

### Traditional (Example-Based)

```rust
#[test]
fn test_reverse() {
    assert_eq!(reverse("hello"), "olleh");
    assert_eq!(reverse("rust"), "tsur");
    assert_eq!(reverse(""), "");
}
```

**Coverage:** 3 specific cases

### Property-Based

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn reverse_inverts(s: String) {
        assert_eq!(reverse(&reverse(&s)), s);
    }
}
```

**Coverage:** Hundreds of generated strings

## Setup

### Add Dependencies

```toml
[dev-dependencies]
proptest = "1.5"
quickcheck = "1.0"
quickcheck_macros = "1.0"
```

## Using Proptest

### Basic Property Test

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_addition_commutative(a: i32, b: i32) {
        // Property: a + b == b + a
        prop_assert_eq!(a + b, b + a);
    }

    #[test]
    fn test_multiplication_associative(a: i32, b: i32, c: i32) {
        // Property: (a * b) * c == a * (b * c)
        prop_assert_eq!((a * b) * c, a * (b * c));
    }
}
```

### Custom Strategies

Generate specific input types:

```rust
use proptest::prelude::*;

fn valid_email() -> impl Strategy<Value = String> {
    "[a-z]{1,20}@[a-z]{1,10}\\.[a-z]{2,3}"
        .prop_map(|s| s.to_string())
}

proptest! {
    #[test]
    fn test_email_parser(email in valid_email()) {
        let parsed = parse_email(&email);
        prop_assert!(parsed.is_ok());
    }
}
```

### Constrained Inputs

```rust
proptest! {
    #[test]
    fn test_positive_division(
        a in 1..1000i32,  // 1 to 999
        b in 1..100i32    // 1 to 99
    ) {
        let result = a / b;
        prop_assert!(result >= 0);
        prop_assert!(result <= a);
    }
}
```

### Shrinking

When a test fails, proptest **shrinks** the input to find minimal failing case:

```rust
proptest! {
    #[test]
    fn test_fails_large_numbers(n: u32) {
        prop_assert!(n < 1000);  // Fails for n >= 1000
    }
}
```

**Output:**
```
Test failed for input: n = 1000
(shrunk from initial failure: n = 4294967295)
```

## Common Properties

### 1. Idempotence

**Property:** Applying operation twice = applying once

```rust
proptest! {
    #[test]
    fn sort_is_idempotent(mut vec: Vec<i32>) {
        vec.sort();
        let first = vec.clone();
        vec.sort();
        prop_assert_eq!(vec, first);
    }
}
```

### 2. Round-Trip (Encode/Decode)

**Property:** Decode(Encode(x)) == x

```rust
proptest! {
    #[test]
    fn serialize_roundtrip(data: MyStruct) {
        let bytes = serialize(&data);
        let decoded = deserialize(&bytes).unwrap();
        prop_assert_eq!(data, decoded);
    }
}
```

### 3. Invariants

**Property:** Certain conditions always hold

```rust
proptest! {
    #[test]
    fn heap_maintains_max(ops: Vec<HeapOp>) {
        let mut heap = MaxHeap::new();
        for op in ops {
            match op {
                HeapOp::Push(n) => heap.push(n),
                HeapOp::Pop => { heap.pop(); }
            }
            // Invariant: top element is maximum
            if let Some(top) = heap.peek() {
                for item in heap.iter() {
                    prop_assert!(*top >= *item);
                }
            }
        }
    }
}
```

### 4. Commutativity

**Property:** Order doesn't matter

```rust
proptest! {
    #[test]
    fn set_insertion_commutative(a: i32, b: i32) {
        let mut set1 = HashSet::new();
        set1.insert(a);
        set1.insert(b);

        let mut set2 = HashSet::new();
        set2.insert(b);
        set2.insert(a);

        prop_assert_eq!(set1, set2);
    }
}
```

### 5. Associativity

**Property:** Grouping doesn't matter

```rust
proptest! {
    #[test]
    fn string_concat_associative(a: String, b: String, c: String) {
        let left = format!("{}{}{}", a, b, c);
        let right = format!("{}{}{}", a, b, c);
        prop_assert_eq!(left, right);
    }
}
```

### 6. Monotonicity

**Property:** Output increases with input

```rust
proptest! {
    #[test]
    fn absolute_value_monotonic(a: i32, b: i32) {
        if a <= b {
            prop_assert!(a.abs() <= b.abs() || a.abs() >= b.abs());
        }
    }
}
```

### 7. Oracle (Test Against Known Implementation)

```rust
proptest! {
    #[test]
    fn optimized_matches_reference(input: Vec<i32>) {
        let optimized = fast_sort(&input);
        let reference = input.clone().sorted();
        prop_assert_eq!(optimized, reference);
    }
}
```

## Using QuickCheck

Alternative to proptest with simpler API:

```rust
use quickcheck_macros::quickcheck;

#[quickcheck]
fn reverse_reverse_is_identity(vec: Vec<i32>) -> bool {
    let mut v = vec.clone();
    v.reverse();
    v.reverse();
    v == vec
}

#[quickcheck]
fn adding_zero_is_identity(n: i32) -> bool {
    n + 0 == n && 0 + n == n
}
```

## Advanced Strategies

### Composite Types

```rust
#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
    age: u8,
}

fn user_strategy() -> impl Strategy<Value = User> {
    (
        any::<u64>(),
        "[a-zA-Z]{1,20}",
        1u8..120u8
    ).prop_map(|(id, name, age)| User {
        id,
        name: name.to_string(),
        age,
    })
}

proptest! {
    #[test]
    fn test_user_validation(user in user_strategy()) {
        prop_assert!(validate_user(&user).is_ok());
    }
}
```

### Recursive Structures

```rust
#[derive(Debug, Clone, PartialEq)]
enum Tree {
    Leaf(i32),
    Node(Box<Tree>, Box<Tree>),
}

fn tree_strategy() -> impl Strategy<Value = Tree> {
    let leaf = any::<i32>().prop_map(Tree::Leaf);
    leaf.prop_recursive(
        8,   // max depth
        256, // max nodes
        10,  // items per collection
        |inner| {
            (inner.clone(), inner).prop_map(|(l, r)| {
                Tree::Node(Box::new(l), Box::new(r))
            })
        },
    )
}
```

### Weighted Strategies

```rust
fn operation_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => Just(Op::Add),     // 30% probability
        3 => Just(Op::Remove),  // 30%
        2 => Just(Op::Clear),   // 20%
        2 => Just(Op::Reset),   // 20%
    ]
}
```

## Configuration

### Number of Test Cases

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn extensive_test(n: u64) {
        // Runs 10,000 test cases instead of default 256
    }
}
```

### Reproducible Failures

```rust
// Failed seed from test output
proptest! {
    #![proptest_config(ProptestConfig {
        rng_algorithm: RngAlgorithm::ChaCha,
        seed: Some([0xdeadbeef; 4]),
        ..Default::default()
    })]

    #[test]
    fn reproducible_test(n: i32) {
        // Uses fixed seed for reproduction
    }
}
```

## Best Practices

### 1. Start Simple

```rust
// Start with basic property
proptest! {
    #[test]
    fn len_after_push(mut vec: Vec<i32>, n: i32) {
        let original_len = vec.len();
        vec.push(n);
        prop_assert_eq!(vec.len(), original_len + 1);
    }
}
```

### 2. Test Properties, Not Implementation

```rust
// ❌ Bad - tests implementation details
proptest! {
    #[test]
    fn uses_quicksort(vec: Vec<i32>) {
        assert!(is_using_quicksort(&vec));
    }
}

// ✅ Good - tests behavior
proptest! {
    #[test]
    fn result_is_sorted(vec: Vec<i32>) {
        let sorted = my_sort(&vec);
        prop_assert!(is_sorted(&sorted));
    }
}
```

### 3. Combine with Example Tests

```rust
// Specific edge cases
#[test]
fn test_empty_vec() {
    assert_eq!(process(&[]), vec![]);
}

// General properties
proptest! {
    #[test]
    fn output_length_matches(input: Vec<i32>) {
        prop_assert_eq!(process(&input).len(), input.len());
    }
}
```

### 4. Use Preconditions

```rust
proptest! {
    #[test]
    fn division_property(a: i32, b: i32) {
        prop_assume!(b != 0);  // Skip if precondition fails
        let result = a / b;
        prop_assert_eq!(result * b, a - (a % b));
    }
}
```

## Common Patterns

### Testing Parsers

```rust
proptest! {
    #[test]
    fn parser_roundtrip(ast: AST) {
        let text = format_ast(&ast);
        let parsed = parse(&text).unwrap();
        prop_assert_eq!(parsed, ast);
    }
}
```

### Testing Data Structures

```rust
proptest! {
    #[test]
    fn map_operations(ops: Vec<MapOp>) {
        let mut map = MyMap::new();
        let mut reference = HashMap::new();

        for op in ops {
            match op {
                MapOp::Insert(k, v) => {
                    map.insert(k, v);
                    reference.insert(k, v);
                }
                MapOp::Remove(k) => {
                    map.remove(&k);
                    reference.remove(&k);
                }
            }
        }

        prop_assert_eq!(map.len(), reference.len());
    }
}
```

### Testing Concurrent Code

```rust
proptest! {
    #[test]
    fn concurrent_counter(ops: Vec<CounterOp>) {
        let counter = Arc::new(AtomicUsize::new(0));
        let handles: Vec<_> = ops.into_iter().map(|op| {
            let c = counter.clone();
            thread::spawn(move || {
                match op {
                    CounterOp::Inc => c.fetch_add(1, Ordering::SeqCst),
                    CounterOp::Dec => c.fetch_sub(1, Ordering::SeqCst),
                }
            })
        }).collect();

        for h in handles {
            h.join().unwrap();
        }

        // Invariant: counter value is consistent
        prop_assert!(counter.load(Ordering::SeqCst) >= 0);
    }
}
```

## Debugging Failures

### Reproduce Specific Case

```rust
#[test]
fn debug_specific_case() {
    // From proptest failure message
    let input = vec![1, 2, 3];
    assert!(my_function(&input));
}
```

### Print Debugging

```rust
proptest! {
    #[test]
    fn debug_test(n: i32) {
        println!("Testing with n = {}", n);
        prop_assert!(n >= 0 || n < 0);
    }
}
```

## Performance Considerations

Property tests are slower than example tests:
- Run 100-1000+ cases vs 1-10 examples
- Generation overhead

**Optimize:**
- Use in CI, not on every save
- Reduce cases for fast feedback: `with_cases(100)`
- Increase cases for thorough testing: `with_cases(10000)`

## Links

- [Proptest Documentation](https://docs.rs/proptest/)
- [QuickCheck Documentation](https://docs.rs/quickcheck/)
- [Property-Based Testing Book](https://fsharpforfunandprofit.com/posts/property-based-testing/)
- [Hypothesis (Python PBT)](https://hypothesis.readthedocs.io/) - Similar concepts
