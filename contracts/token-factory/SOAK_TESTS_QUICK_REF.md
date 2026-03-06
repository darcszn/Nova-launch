# Soak Tests Quick Reference

## Run Commands

```bash
# Run all soak tests
cargo test soak_test --ignored -- --nocapture

# Run specific test
cargo test soak_test_mint_operations --ignored -- --nocapture
cargo test soak_test_burn_operations --ignored -- --nocapture
cargo test soak_test_mixed_operations --ignored -- --nocapture
cargo test soak_test_stress_single_token --ignored -- --nocapture
cargo test soak_test_pause_unpause_cycles --ignored -- --nocapture
```

## Test Summary

| Test | Operations | Focus | Duration |
|------|-----------|-------|----------|
| `soak_test_mint_operations` | 100k mints | Supply growth | ~5-10 min |
| `soak_test_burn_operations` | 100k burns | Supply reduction | ~5-10 min |
| `soak_test_mixed_operations` | 100k mixed | All operations | ~10-15 min |
| `soak_test_stress_single_token` | 100k on 1 token | High contention | ~8-12 min |
| `soak_test_pause_unpause_cycles` | 100k with pause | State transitions | ~10-15 min |

## Invariants

✅ **Supply Conservation**: `total_supply + total_burned = initial_supply`  
✅ **Balance Sum**: `sum(all_balances) = total_supply`  
✅ **Non-Negativity**: All balances >= 0  
✅ **Metadata Immutable**: Once set, never changes  
✅ **Creator Immutable**: Never changes  

## Configuration

Edit in `contracts/token-factory/src/soak_tests.rs`:

```rust
const SOAK_TEST_OPERATIONS: u32 = 100_000;  // Total operations
const TOKENS_TO_CREATE: u32 = 10;           // Number of tokens
const HOLDERS_PER_TOKEN: u32 = 20;          // Holders per token
```

## Checkpoint Intervals

- Mint test: Every 10k operations
- Burn test: Every 10k operations
- Mixed test: Every 10k operations
- Stress test: Every 5k operations (more frequent)
- Pause test: Every 10k operations

## Expected Output

```
Starting soak test: 100000 mint operations
Checkpoint: 10000 operations completed
Checkpoint: 20000 operations completed
...
✅ All invariants maintained after 100000 operations
```

## Troubleshooting

**Timeout**: Reduce `SOAK_TEST_OPERATIONS`  
**Memory**: Reduce `TOKENS_TO_CREATE` or `HOLDERS_PER_TOKEN`  
**Invariant Failure**: Check operation number and specific invariant in error message

## Files

- `contracts/token-factory/src/soak_tests.rs` - Test implementation
- `contracts/token-factory/SOAK_TESTS_GUIDE.md` - Detailed guide
- `contracts/token-factory/SOAK_TESTS_QUICK_REF.md` - This file
