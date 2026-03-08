# Security Testing Guide

## Overview

This guide explains how to run, maintain, and extend the security test suite that validates our threat model coverage.

## Quick Start

```bash
# Run all security tests
npm run test:security

# Run specific category
npm run test:security:auth
npm run test:security:arithmetic
npm run test:security:timing
npm run test:security:events
npm run test:security:dos

# Generate coverage report
npm run test:security:coverage
```

## Test Structure

### Test Files

| File | Risk Category | Severity | Test Count |
|------|---------------|----------|------------|
| `security.auth.bypass.test.ts` | Authentication Bypass | CRITICAL | 25+ |
| `security.auth.replay.test.ts` | Replay Attacks | CRITICAL | 20+ |
| `security.arithmetic.test.ts` | Arithmetic Faults | HIGH | 25+ |
| `security.timing.test.ts` | Race Conditions | HIGH | 20+ |
| `security.event.tampering.test.ts` | Event Injection | HIGH | 30+ |
| `security.dos.test.ts` | Resource Exhaustion | MEDIUM | 15+ |

### Risk Coverage Matrix

Each test file includes a header documenting which risks it covers:

```typescript
/**
 * SECURITY TEST: Authentication & Authorization Bypass
 * 
 * RISK COVERAGE:
 * - AUTH-004: JWT token reuse after revocation
 * - AUTH-005: Refresh token type confusion
 * - AUTH-006: Missing JWT signature validation
 * 
 * SEVERITY: CRITICAL
 */
```

## Test Categories

### 1. Authentication & Authorization (CRITICAL)

**Risks Covered:**
- AUTH-001 through AUTH-008
- REPLAY-001, REPLAY-004

**Key Tests:**
- Nonce replay prevention
- Token revocation enforcement
- Signature validation
- Public key format validation

**Example:**
```typescript
it('should reject reused nonce', () => {
  const { nonce } = nonceService.generateNonce(publicKey);
  
  // First use succeeds
  expect(nonceService.consumeNonce(nonce, publicKey)).toBe(true);
  
  // Replay fails
  expect(nonceService.consumeNonce(nonce, publicKey)).toBe(false);
});
```

### 2. Arithmetic Operations (HIGH)

**Risks Covered:**
- ARITH-001 through ARITH-005

**Key Tests:**
- Integer overflow handling
- Negative value rejection
- BigInt precision preservation
- Division by zero prevention
- Balance underflow protection

**Example:**
```typescript
it('should handle MAX_SAFE_INTEGER without overflow', () => {
  const weight = BigInt(Number.MAX_SAFE_INTEGER);
  const doubled = weight + weight;
  
  expect(doubled).toBe(BigInt(Number.MAX_SAFE_INTEGER) * BigInt(2));
});
```

### 3. Timing & Race Conditions (HIGH)

**Risks Covered:**
- TIMING-001 through TIMING-005
- DOS-001

**Key Tests:**
- Concurrent nonce consumption
- Double-spend prevention
- Token revocation races
- Cache timing attacks
- Rate limit bypass

**Example:**
```typescript
it('should prevent race condition in nonce consumption', async () => {
  const { nonce } = nonceService.generateNonce(publicKey);
  
  // 10 concurrent attempts
  const attempts = Array(10).fill(null)
    .map(() => nonceService.consumeNonce(nonce, publicKey));
  
  const results = await Promise.all(attempts);
  
  // Only one succeeds
  expect(results.filter(r => r === true).length).toBe(1);
});
```

### 4. Event Tampering (HIGH)

**Risks Covered:**
- EVENT-001 through EVENT-006
- REPLAY-002, REPLAY-003

**Key Tests:**
- Malicious event injection
- Field manipulation
- Event ordering validation
- Webhook signature verification
- SQL injection prevention
- XSS sanitization

**Example:**
```typescript
it('should reject event with invalid type', async () => {
  const maliciousEvent = {
    type: 'MALICIOUS_EVENT_TYPE',
    data: { proposalId: 1 }
  };
  
  await expect(parser.parseEvent(maliciousEvent)).rejects.toThrow();
});
```

### 5. Resource Exhaustion (MEDIUM)

**Risks Covered:**
- DOS-002, DOS-003

**Key Tests:**
- Query result size limits
- Payload size validation
- Cache poisoning prevention
- Connection limits
- Memory exhaustion protection

**Example:**
```typescript
it('should limit query result size', () => {
  const maxLimit = 100;
  const requestedLimit = 10000;
  
  const actualLimit = Math.min(requestedLimit, maxLimit);
  expect(actualLimit).toBe(maxLimit);
});
```

## Coverage Requirements

### Critical Risks
- **Required Coverage:** 100%
- **No Skipped Tests:** All tests must run
- **CI Enforcement:** Blocks merge if failing

### High Risks
- **Required Coverage:** 95%
- **Allowed Skips:** None
- **CI Enforcement:** Blocks merge if failing

### Medium Risks
- **Required Coverage:** 80%
- **Allowed Skips:** With justification
- **CI Enforcement:** Warning only

## CI Integration

### Automated Checks

The CI pipeline runs on every PR and includes:

1. **Test Execution:** All security tests must pass
2. **Coverage Validation:** Thresholds must be met
3. **Risk Matrix Validation:** All critical risks have tests
4. **Documentation Check:** All tests properly documented
5. **No Skipped Tests:** No `.skip` or `.todo` in critical tests

### Workflow File

See `.github/workflows/security-tests.yml` for the complete CI configuration.

### PR Requirements

Before merging, PRs must:
- ✅ Pass all security tests
- ✅ Meet coverage thresholds
- ✅ Have no skipped critical tests
- ✅ Update risk matrix if adding new risks
- ✅ Document new test coverage

## Adding New Tests

### Step 1: Identify Risk

Check `THREAT_MODEL_RISK_MATRIX.md` for uncovered risks or add new risk:

```markdown
| RISK-NEW | Description | Attack Vector | Test Coverage |
|----------|-------------|---------------|---------------|
| AUTH-009 | New auth risk | Attack method | `security.auth.bypass.test.ts` |
```

### Step 2: Write Test

Add test to appropriate file with risk documentation:

```typescript
describe('[AUTH-009] New Auth Risk', () => {
  it('should prevent new attack vector', () => {
    // Test implementation
  });
});
```

### Step 3: Update Documentation

1. Add risk to `THREAT_MODEL_RISK_MATRIX.md`
2. Update test file header with new risk ID
3. Update this guide if new category

### Step 4: Verify CI

```bash
# Run locally first
npm run test:security

# Check coverage
npm run test:security:coverage

# Verify CI will pass
npm run test:security:auth  # Or relevant category
```

## Best Practices

### Test Isolation

Each test should be independent:

```typescript
beforeEach(() => {
  // Fresh instances for each test
  nonceService = new NonceService();
});

afterEach(() => {
  // Cleanup
  nonceService.onModuleDestroy();
});
```

### Realistic Attack Scenarios

Test real-world attack patterns:

```typescript
it('should prevent timing attack on signature comparison', () => {
  // Use timing-safe comparison
  const timingSafeEqual = (a: string, b: string): boolean => {
    if (a.length !== b.length) return false;
    
    let result = 0;
    for (let i = 0; i < a.length; i++) {
      result |= a.charCodeAt(i) ^ b.charCodeAt(i);
    }
    return result === 0;
  };
  
  // Test implementation
});
```

### Clear Failure Messages

Make failures easy to diagnose:

```typescript
expect(result).toBe(expected);  // ❌ Unclear

expect(result).toBe(expected);  // ✅ With context
// "Expected nonce consumption to fail on replay attack"
```

### Document Attack Vectors

Explain what you're testing:

```typescript
it('should prevent TOCTOU in token validation', () => {
  // Time-of-check: Token is valid
  const isValidAtCheck = !tokenService.isRevoked(jti);
  
  // Attacker revokes between check and use
  tokenService.revokeToken(jti);
  
  // Time-of-use: Must check again
  const isValidAtUse = !tokenService.isRevoked(jti);
  expect(isValidAtUse).toBe(false);
});
```

## Troubleshooting

### Tests Failing Locally

```bash
# Clear test cache
rm -rf node_modules/.vitest

# Reinstall dependencies
npm ci

# Run with verbose output
npm run test:security -- --reporter=verbose
```

### Coverage Not Meeting Threshold

```bash
# Generate detailed coverage report
npm run test:security:coverage

# Open HTML report
open coverage/index.html
```

### CI Failing But Local Passing

- Check Node.js version matches CI (18.x or 20.x)
- Verify environment variables
- Check for timing-dependent tests
- Review CI logs for specific failures

## Maintenance

### Quarterly Review

Every 3 months:
1. Review threat model for new risks
2. Update risk matrix
3. Add tests for new attack vectors
4. Review and update coverage thresholds
5. Penetration testing validation

### After Security Incidents

1. Add test reproducing the vulnerability
2. Verify fix prevents the attack
3. Update threat model
4. Document lessons learned

### Dependency Updates

When updating security-related dependencies:
1. Run full security test suite
2. Review breaking changes
3. Update tests if APIs changed
4. Verify coverage maintained

## Resources

- [THREAT_MODEL_RISK_MATRIX.md](./THREAT_MODEL_RISK_MATRIX.md) - Complete risk mapping
- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [NIST Security Testing](https://csrc.nist.gov/projects/security-testing)

## Support

For questions or issues:
- Security Team: security@example.com
- Slack: #security-testing
- Wiki: [Security Testing](https://wiki.example.com/security-testing)
