# mint_tokens Quick Reference

## Function Signature

```rust
pub fn mint_tokens(
    env: Env,
    token_index: u32,
    admin: Address,
    to: Address,
    amount: i128,
) -> Result<(), Error>
```

## Purpose

Allows token creators to mint additional tokens after deployment, increasing the total supply.

## Security Checks (in order)

1. Contract not paused
2. Admin authorization (`require_auth()`)
3. Amount > 0
4. Token exists
5. Admin is token creator
6. No arithmetic overflow

## Error Codes

| Error | When |
|-------|------|
| `ContractPaused` | Contract is paused |
| `TokenNotFound` | Invalid token index |
| `Unauthorized` | Caller is not creator |
| `InvalidAmount` | Amount ≤ 0 |
| `ArithmeticError` | Overflow would occur |

## Usage

```rust
// Mint 1,000,000 tokens to recipient
let amount = 1_000_000_0000000i128;
factory.mint_tokens(&0, &creator, &recipient, &amount)?;
```

## Effects

- ✅ Increases recipient balance by `amount`
- ✅ Increases total supply by `amount`
- ✅ Updates storage (index + address lookups)
- ✅ Emits `tok_mint` event

## Event Emitted

```
Symbol: "tok_mint"
Indexed: token_address
Payload: (admin, to, amount)
```

## Key Features

- **Unlimited**: Can mint multiple times
- **Creator-only**: Only token creator can mint
- **Pausable**: Respects contract pause state
- **Safe**: Overflow protection with checked arithmetic
- **Atomic**: All storage updates together
- **Auditable**: Emits event for tracking

## Common Use Cases

1. **Initial Distribution**: Mint to team/investors
2. **Rewards**: Mint tokens as rewards
3. **Supply Expansion**: Increase circulating supply
4. **Airdrops**: Distribute to multiple recipients

## Comparison with Burn

| Feature | mint_tokens | burn |
|---------|-------------|------|
| **Effect** | Increases supply | Decreases supply |
| **Authorization** | Creator only | Holder or admin |
| **Amount** | Must be > 0 | Must be > 0 |
| **Overflow** | Checked | N/A |
| **Underflow** | N/A | Checked |

## Best Practices

✅ **DO**:
- Mint in reasonable amounts
- Document minting rationale
- Monitor total supply
- Use events for transparency

❌ **DON'T**:
- Mint without clear purpose
- Ignore overflow risks
- Mint while paused
- Attempt unauthorized minting

## Gas Cost

Approximate: **~9400 units**

Breakdown:
- Authorization: ~500
- Validation: ~150
- Storage reads: ~2000
- Storage writes: ~6000
- Event emission: ~500
- Calculations: ~250
