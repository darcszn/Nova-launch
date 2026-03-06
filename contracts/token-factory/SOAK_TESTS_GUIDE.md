# Long-Run Soak Tests Guide

## Overview

This guide covers the comprehensive soak test suite that validates contract behavior under heavy load with 100,000+ operations. These tests ensure state consistency, prevent resource exhaustion, and verify that critical invariants hold across arbitrary operation sequences.

## Test Suite

### 1. Mint Operations Soak Test
**Test**: `soak_test_mint_operations`  
**Operations**: 100,000 mint operations  
**Tokens**: 10 tokens  
**Holders**: 20 holders per token  

Validates that continuous minting operations maintain:
- Supply conservation
- Balance non-negativity
- No arithmetic overflows
- Consistent state across checkpoints

**Run**:
```bash
cargo test soak_test_mint_operations --ignored -- --nocapture
```

### 2. Burn Operations Soak Test
**Test**: `soak_test_burn_operations`  
**Operations**: 100,000 burn operations  
**Tokens**: 10 tokens  
**Holders**: 20 holders per token (each with 10M initial balance)  

Validates that continuous burning operations maintain:
- Supply conservation (supply + burned = initial)
- Balance non-negativity
- Proper burn count tracking
- No underflow errors

**Run**:
```bash
cargo test soak_test_burn_operations --ignored -- --nocapture
```

### 3. Mixed Operations Soak Test
**Test**: `soak_test_mixed_operations`  
**Operations**: 100,000 mixed operations (mint, burn, set_metadata, admin_burn, queries)  
**Tokens**: 10 tokens  
**Holders**: 20 holders per token  

Validates that mixed operation sequences maintain:
- All supply invariants
- Metadata immutability (once set, never changes)
- Creator immutability (never changes)
- Consistent state across all operation types

**Run**:
```bash
cargo test soak_test_mixed_operations --ignored -- --nocapture
```

### 4. Stress Single Token Test
**Test**: `soak_test_stress_single_token`  
**Operations**: 100,000 operations on a single token  
**Tokens**: 1 token  
**Holders**: 100 holders  

Validates that intensive operations on a single token maintain:
- All invariants under high contention
- No performance degradation
- Consistent state with many concurrent holders

**Run**:
```bash
cargo test soak_test_stress_single_token --ignored -- --nocapture
```

### 5. Pause/Unpause Cycles Test
**Test**: `soak_test_pause_unpause_cycles`  
**Operations**: 100,000 operations with frequent pause/unpause  
**Tokens**: 1 token  
**Holders**: 20 holders  

Validates that pause/unpause cycles:
- Properly block operations when paused
- Resume correctly when unpaused
- Maintain all invariants across state transitions
- Track successful vs blocked operations

**Run**:
```bash
cargo test soak_test_pause_unpause_cycles --ignored -- --nocapture
```

## Invariants Checked

### 1. Supply Conservation
**Rule**: `total_supply + total_burned = initial_supply`  
**Also**: `sum(all_balances) = total_supply`

This ensures no tokens are created or destroyed outside of mint/burn operations.

### 2. Balance Non-Negativity
**Rule**: All balances >= 0

Ensures no underflow errors in balance tracking.

### 3. Supply Monotonicity
**Rule**: Supply only increases via mint, decreases via burn

Ensures predictable supply changes.

### 4. Metadata Immutability
**Rule**: Once set, metadata never changes

Critical for trust and data integrity.

### 5. Creator Immutability
**Rule**: Token creator never changes

Ensures consistent ownership and permissions.

### 6. Token Count Monotonicity
**Rule**: Token count only increases

Ensures consistent token indexing.

## Checkpoint System

All soak tests implement checkpoint intervals (every 10,000 operations) where:
1. All invariants are verified
2. Current state is logged
3. Test fails immediately if any invariant is violated

This provides early detection of issues and detailed failure information.

## Running All Soak Tests

Run all soak tests together:
```bash
cargo test --test '*' --ignored -- --nocapture soak_test
```

Run with specific test filter:
```bash
cargo test soak_test --ignored -- --nocapture
```

## Performance Expectations

- Each test should complete in 5-15 minutes depending on hardware
- Memory usage should remain stable (no leaks)
- CPU usage should be consistent (no degradation)

## Interpreting Results

### Success Output
```
Starting soak test: 100000 mint operations
Checkpoint: 10000 operations completed
Checkpoint: 20000 operations completed
...
Checkpoint: 100000 operations completed
Completed 100000 mint operations successfully
Performing final invariant checks...
Token 0: supply=1500000000000, burned=0, burn_count=0
...
✅ All invariants maintained after 100000 operations
```

### Failure Output
If an invariant is violated, you'll see:
```
Checkpoint: 45000 operations completed
thread 'soak_test_mint_operations' panicked at 'Invariant violation at operation 45000: 
Supply conservation violated: balance_sum=1000000, total_supply=1000001'
```

This indicates:
- Which operation number caused the failure
- Which invariant was violated
- The specific values that don't match

## Troubleshooting

### Test Timeout
If tests timeout, reduce `SOAK_TEST_OPERATIONS` constant in `soak_tests.rs`.

### Memory Issues
If you encounter memory issues:
1. Reduce `TOKENS_TO_CREATE` or `HOLDERS_PER_TOKEN`
2. Run tests individually instead of all at once
3. Increase checkpoint interval to reduce logging overhead

### Invariant Failures
If invariants fail:
1. Note the operation number where failure occurred
2. Check the specific invariant that failed
3. Review recent changes to related functions
4. Add additional logging around the failing operation type

## Integration with CI/CD

These tests are marked with `#[ignore]` to prevent them from running in standard CI pipelines. To include them in CI:

```yaml
# .github/workflows/soak-tests.yml
name: Soak Tests
on:
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sunday at 2 AM
  workflow_dispatch:

jobs:
  soak-tests:
    runs-on: ubuntu-latest
    timeout-minutes: 120
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Soak Tests
        run: cargo test --ignored -- --nocapture soak_test
```

## Adding New Soak Tests

To add a new soak test:

1. Create test function with `#[test]` and `#[ignore]` attributes
2. Use `setup_soak_test()` helper for environment setup
3. Create tokens and holders as needed
4. Implement operation loop with checkpoint intervals
5. Use `InvariantChecker` to verify state at checkpoints
6. Add final comprehensive invariant check
7. Document the test in this guide

Example template:
```rust
#[test]
#[ignore]
fn soak_test_my_new_test() {
    let env = Env::default();
    let (client, admin, _treasury) = setup_soak_test(&env);
    
    // Setup tokens and holders
    
    let checker = InvariantChecker::new(&env, &client);
    
    for i in 0..SOAK_TEST_OPERATIONS {
        // Perform operations
        
        if i % 10_000 == 0 {
            // Checkpoint: verify invariants
            checker.check_token_invariants(token_index, &holders)
                .expect("Invariant check failed");
        }
    }
    
    // Final check
    println!("✅ Test completed successfully");
}
```

## Best Practices

1. **Always use checkpoints**: Verify invariants regularly, not just at the end
2. **Log progress**: Use `println!` to track test progress
3. **Test realistic scenarios**: Mix operations as they would occur in production
4. **Vary parameters**: Test with different token counts, holder counts, amounts
5. **Document expectations**: Clearly state what each test validates
6. **Handle edge cases**: Include boundary conditions in operation sequences

## Related Documentation

- `BURN_TESTS_IMPLEMENTATION.md` - Burn operation tests
- `SET_METADATA_IMPLEMENTATION.md` - Metadata tests
- `MINT_TOKENS_IMPLEMENTATION.md` - Mint operation tests
- `PERFORMANCE_TESTING_COMPLETE.md` - Performance benchmarks
