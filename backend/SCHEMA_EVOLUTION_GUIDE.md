# Schema Evolution and Backward Compatibility Guide

## Overview

This guide documents the schema evolution strategy for the Nova Launch backend, ensuring backward compatibility when introducing new fields and keys in storage models.

## Schema Evolution Timeline

### Token Model

| Version | Date | Changes | Migration Required |
|---------|------|---------|-------------------|
| V1 | 2023-01-01 | Initial schema | N/A |
| V2 | 2023-06-01 | Added `totalBurned`, `burnCount` | No (defaults provided) |
| V3 | 2024-01-01 | Added `metadataUri` | No (nullable field) |

### Stream Model

| Version | Date | Changes | Migration Required |
|---------|------|---------|-------------------|
| V1 | 2023-01-01 | Initial schema | N/A |
| V2 | 2023-06-01 | Added `metadata` | No (nullable field) |
| V3 | 2024-01-01 | Added `claimedAt`, `cancelledAt` | No (nullable fields) |

### Proposal Model

| Version | Date | Changes | Migration Required |
|---------|------|---------|-------------------|
| V1 | 2023-01-01 | Initial schema | N/A |
| V2 | 2023-06-01 | Added `description`, `metadata` | No (nullable fields) |
| V3 | 2024-01-01 | Added `executedAt`, `cancelledAt` | No (nullable fields) |

### Vote Model

| Version | Date | Changes | Migration Required |
|---------|------|---------|-------------------|
| V1 | 2023-01-01 | Initial schema | N/A |
| V2 | 2023-06-01 | Added `reason` | No (nullable field) |

### BurnRecord Model

| Version | Date | Changes | Migration Required |
|---------|------|---------|-------------------|
| V1 | 2023-01-01 | Initial schema | N/A |
| V2 | 2023-06-01 | Added `isAdminBurn` | No (default: false) |

## Field Default Values

### Token Model

```typescript
{
  decimals: 18,           // Default decimal places
  totalBurned: 0,         // Default to no burns
  burnCount: 0,           // Default to no burn count
  metadataUri: null,      // Optional metadata
}
```

### Stream Model

```typescript
{
  status: 'CREATED',      // Default status
  metadata: null,         // Optional metadata
  claimedAt: null,        // Set when claimed
  cancelledAt: null,      // Set when cancelled
}
```

### Proposal Model

```typescript
{
  status: 'ACTIVE',       // Default status
  description: null,      // Optional description
  metadata: null,         // Optional metadata
  executedAt: null,       // Set when executed
  cancelledAt: null,      // Set when cancelled
}
```

### Vote Model

```typescript
{
  reason: null,           // Optional voting reason
}
```

### BurnRecord Model

```typescript
{
  isAdminBurn: false,     // Default to user burn
}
```

## Backward Compatibility Guarantees

### 1. Reading Old Data

All old data remains readable after schema upgrades:

```typescript
// V1 token (no totalBurned, burnCount, metadataUri)
const v1Token = await prisma.token.findUnique({
  where: { address: 'COLD_TOKEN' }
});

// New fields will have default values
expect(v1Token.totalBurned).toBe(BigInt(0));
expect(v1Token.burnCount).toBe(0);
expect(v1Token.metadataUri).toBeNull();
```

### 2. Writing to Old Data

Old data can be updated with new fields without breaking:

```typescript
// Update V1 token with V3 fields
await prisma.token.update({
  where: { address: 'COLD_TOKEN' },
  data: {
    totalBurned: BigInt('100000000000'),
    burnCount: 10,
    metadataUri: 'ipfs://QmNewMetadata',
  },
});
```

### 3. Mixed Version Data

Different versions of data can coexist in the same table:

```typescript
// V1, V2, and V3 tokens can exist simultaneously
const allTokens = await prisma.token.findMany();

// Each will have appropriate defaults for missing fields
```

## Migration Strategies

### Strategy 1: Lazy Migration

New fields are added as nullable or with defaults. No immediate migration required.

**Pros:**
- Zero downtime
- No data migration needed
- Gradual adoption

**Cons:**
- Mixed data versions in database
- Need to handle null values

**Example:**
```typescript
// Add new field to schema
model Token {
  // ... existing fields
  metadataUri String? // Nullable, no migration needed
}
```

### Strategy 2: Eager Migration

All existing records are updated with new field values.

**Pros:**
- Consistent data state
- Simpler queries

**Cons:**
- Requires downtime or careful coordination
- Can be slow for large datasets

**Example:**
```typescript
// Migrate all tokens to have metadata
await prisma.token.updateMany({
  where: { metadataUri: null },
  data: { metadataUri: 'ipfs://default' },
});
```

### Strategy 3: Hybrid Migration

Critical fields are migrated eagerly, optional fields lazily.

**Pros:**
- Balance between consistency and performance
- Flexible approach

**Cons:**
- More complex migration logic

## Testing Strategy

### 1. Schema Evolution Tests

Test that old data remains readable:

```typescript
describe('Schema Evolution', () => {
  it('should read V1 token data', async () => {
    // Create V1 token (without new fields)
    const v1Token = await prisma.token.create({
      data: { /* V1 fields only */ },
    });

    // Verify new fields have defaults
    expect(v1Token.totalBurned).toBe(BigInt(0));
  });
});
```

### 2. Migration Tests

Test the migration process:

```typescript
describe('Migration', () => {
  it('should migrate V1 to V3', async () => {
    // Create V1 data
    const v1Token = await createV1Token();

    // Migrate to V2
    await migrateToV2(v1Token);

    // Migrate to V3
    await migrateToV3(v1Token);

    // Verify all data intact
    expect(v1Token.address).toBe(originalAddress);
  });
});
```

### 3. Regression Tests

Test that new fields don't break existing functionality:

```typescript
describe('Regression', () => {
  it('should maintain defaults after update', async () => {
    const token = await prisma.token.create({
      data: { /* minimal fields */ },
    });

    // Update without new fields
    await prisma.token.update({
      where: { id: token.id },
      data: { name: 'Updated' },
    });

    // Verify defaults maintained
    expect(token.totalBurned).toBe(BigInt(0));
  });
});
```

## Best Practices

### 1. Always Provide Defaults

New fields should have sensible defaults:

```prisma
model Token {
  totalBurned BigInt @default(0)  // ✅ Good
  burnCount   Int    @default(0)  // ✅ Good
  metadataUri String?              // ✅ Good (nullable)
}
```

### 2. Use Nullable for Optional Data

Optional fields should be nullable:

```prisma
model Proposal {
  description String?  // ✅ Optional description
  metadata    String?  // ✅ Optional metadata
}
```

### 3. Document Schema Changes

Always document what changed and why:

```typescript
/**
 * V3 Schema Changes (2024-01-01)
 * 
 * Added metadataUri field to support IPFS metadata
 * - Nullable field, no migration required
 * - Existing tokens will have null metadataUri
 */
```

### 4. Test All Migration Paths

Test every version transition:

```typescript
// Test V1 → V2
// Test V2 → V3
// Test V1 → V3 (skip V2)
```

### 5. Maintain Referential Integrity

Ensure relationships work across versions:

```typescript
// V1 token with V2 burn records should work
const token = await prisma.token.findUnique({
  where: { id: v1TokenId },
  include: { burnRecords: true }, // V2 feature
});
```

## Common Pitfalls

### 1. Breaking Changes

❌ **Don't remove or rename fields:**

```prisma
// BAD: Renaming field
model Token {
  // old: totalSupply
  supply BigInt  // ❌ Breaks existing code
}
```

✅ **Add new field, deprecate old:**

```prisma
model Token {
  totalSupply BigInt  // Keep for compatibility
  supply      BigInt  // New field
}
```

### 2. Required Fields Without Defaults

❌ **Don't add required fields without defaults:**

```prisma
model Token {
  newField String  // ❌ Breaks existing data
}
```

✅ **Make it optional or provide default:**

```prisma
model Token {
  newField String?              // ✅ Nullable
  // OR
  newField String @default("")  // ✅ Default value
}
```

### 3. Changing Field Types

❌ **Don't change field types:**

```prisma
// BAD: Changing type
model Token {
  // old: amount BigInt
  amount String  // ❌ Breaks existing data
}
```

✅ **Add new field with new type:**

```prisma
model Token {
  amount       BigInt   // Keep old
  amountString String?  // New representation
}
```

## Rollback Strategy

### 1. Additive Changes Only

Only add fields, never remove:
- Easy to rollback (just ignore new fields)
- Old code continues to work

### 2. Feature Flags

Use feature flags for new fields:

```typescript
if (featureFlags.useMetadata) {
  token.metadataUri = metadata;
}
```

### 3. Gradual Rollout

Deploy schema changes gradually:
1. Add new fields (nullable)
2. Deploy code that can read new fields
3. Deploy code that writes new fields
4. Backfill old data (optional)

## Monitoring

### 1. Track Schema Versions

Log which schema version each record uses:

```typescript
const schemaVersion = token.metadataUri ? 'v3' : 
                     token.totalBurned ? 'v2' : 'v1';
```

### 2. Monitor Migration Progress

Track how many records have been migrated:

```typescript
const v1Count = await prisma.token.count({
  where: { metadataUri: null, totalBurned: 0 }
});
```

### 3. Alert on Anomalies

Alert if unexpected data patterns emerge:

```typescript
// Alert if too many null values
const nullCount = await prisma.token.count({
  where: { metadataUri: null }
});

if (nullCount / totalCount > 0.5) {
  alert('High percentage of null metadata');
}
```

## Conclusion

By following these guidelines, we ensure:
- ✅ Old data remains readable and writable
- ✅ No key collisions or decoding failures
- ✅ Smooth schema evolution without downtime
- ✅ Comprehensive test coverage for all scenarios

All schema changes are tested with the test suites in:
- `src/__tests__/schemaEvolution.test.ts`
- `src/__tests__/migrationUpgrade.test.ts`
- `src/__tests__/fieldDefaultsRegression.test.ts`
