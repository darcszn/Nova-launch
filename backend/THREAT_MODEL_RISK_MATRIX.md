# Threat Model Risk Matrix

## Overview
This document maps security risks identified in the threat model to executable tests, ensuring comprehensive coverage of high and critical security scenarios.

## Risk Categories

### 1. Authentication & Authorization Bypass
**Severity**: CRITICAL

| Risk ID | Description | Attack Vector | Test Coverage |
|---------|-------------|---------------|---------------|
| AUTH-001 | Nonce replay attack | Reusing consumed nonce | `security.auth.replay.test.ts` |
| AUTH-002 | Expired nonce acceptance | Using expired nonce | `security.auth.replay.test.ts` |
| AUTH-003 | Nonce for wrong public key | Nonce/key mismatch | `security.auth.replay.test.ts` |
| AUTH-004 | JWT token reuse after revocation | Using revoked token | `security.auth.bypass.test.ts` |
| AUTH-005 | Refresh token type confusion | Using refresh as access | `security.auth.bypass.test.ts` |
| AUTH-006 | Missing JWT signature validation | Unsigned/invalid JWT | `security.auth.bypass.test.ts` |
| AUTH-007 | Weak signature verification | Invalid Stellar signature | `security.auth.bypass.test.ts` |
| AUTH-008 | Public key format bypass | Malformed public key | `security.auth.bypass.test.ts` |

### 2. Arithmetic & Overflow Faults
**Severity**: HIGH

| Risk ID | Description | Attack Vector | Test Coverage |
|---------|-------------|---------------|---------------|
| ARITH-001 | Integer overflow in vote weight | MAX_SAFE_INTEGER + 1 | `security.arithmetic.test.ts` |
| ARITH-002 | Negative value injection | Negative amounts/weights | `security.arithmetic.test.ts` |
| ARITH-003 | Precision loss in BigInt conversion | Large number handling | `security.arithmetic.test.ts` |
| ARITH-004 | Division by zero | Zero denominators | `security.arithmetic.test.ts` |
| ARITH-005 | Underflow in balance calculations | Subtraction below zero | `security.arithmetic.test.ts` |

### 3. Timing & Race Conditions
**Severity**: HIGH

| Risk ID | Description | Attack Vector | Test Coverage |
|---------|-------------|---------------|---------------|
| TIMING-001 | Concurrent nonce consumption | Parallel nonce use | `security.timing.test.ts` |
| TIMING-002 | Double-spend in vote casting | Concurrent vote submissions | `security.timing.test.ts` |
| TIMING-003 | Token revocation race | Revoke during active use | `security.timing.test.ts` |
| TIMING-004 | Cache invalidation timing | Stale cache exploitation | `security.timing.test.ts` |
| TIMING-005 | Rate limit bypass via timing | Concurrent request bursts | `security.timing.test.ts` |

### 4. Replay Attacks
**Severity**: CRITICAL

| Risk ID | Description | Attack Vector | Test Coverage |
|---------|-------------|---------------|---------------|
| REPLAY-001 | Transaction replay | Resubmit signed transaction | `security.auth.replay.test.ts` |
| REPLAY-002 | Event replay in ingestion | Duplicate event processing | `security.event.tampering.test.ts` |
| REPLAY-003 | Webhook payload replay | Resend webhook with old signature | `security.event.tampering.test.ts` |
| REPLAY-004 | Cross-chain replay | Same tx on different network | `security.auth.replay.test.ts` |

### 5. Event Tampering & Injection
**Severity**: HIGH

| Risk ID | Description | Attack Vector | Test Coverage |
|---------|-------------|---------------|---------------|
| EVENT-001 | Malicious event injection | Crafted governance event | `security.event.tampering.test.ts` |
| EVENT-002 | Event field manipulation | Modified event data | `security.event.tampering.test.ts` |
| EVENT-003 | Event ordering manipulation | Out-of-order events | `security.event.tampering.test.ts` |
| EVENT-004 | Webhook signature bypass | Invalid/missing signature | `security.event.tampering.test.ts` |
| EVENT-005 | SQL injection via event data | Malicious SQL in fields | `security.event.tampering.test.ts` |
| EVENT-006 | XSS via event metadata | Script injection in strings | `security.event.tampering.test.ts` |

### 6. Rate Limiting & DoS
**Severity**: MEDIUM

| Risk ID | Description | Attack Vector | Test Coverage |
|---------|-------------|---------------|---------------|
| DOS-001 | Rate limit bypass | Distributed requests | `security.timing.test.ts` |
| DOS-002 | Resource exhaustion | Large payload/query | `security.dos.test.ts` |
| DOS-003 | Cache poisoning | Malicious cache entries | `security.dos.test.ts` |

## Test Coverage Requirements

### Critical Risks (Must Have 100% Coverage)
- All AUTH-* risks
- All REPLAY-* risks

### High Risks (Must Have 95% Coverage)
- All ARITH-* risks
- All TIMING-* risks
- All EVENT-* risks

### Medium Risks (Must Have 80% Coverage)
- All DOS-* risks

## CI Enforcement

The CI pipeline enforces:
1. All high/critical risk tests must pass
2. No skipped tests for critical risks
3. Coverage thresholds must be met
4. Security test suite runs on every PR

## Test Execution

```bash
# Run all security tests
npm run test:security

# Run specific risk category
npm run test:security:auth
npm run test:security:arithmetic
npm run test:security:timing
npm run test:security:events

# Generate coverage report
npm run test:security:coverage
```

## Traceability Matrix

Each test file includes a header mapping tests to risk IDs:

```typescript
/**
 * RISK COVERAGE:
 * - AUTH-001: Nonce replay attack
 * - AUTH-002: Expired nonce acceptance
 * - AUTH-003: Nonce/key mismatch
 */
```

## Review Process

1. New risks must be added to this matrix
2. Tests must be written before marking risk as covered
3. Security team reviews quarterly
4. Penetration testing validates coverage annually

## Last Updated
- Date: 2024-03-07
- Reviewer: Security Team
- Next Review: 2024-06-07
