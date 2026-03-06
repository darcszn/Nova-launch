# Stream Metadata Feature

## Overview
Added optional metadata support for payment streams to enable discoverability through labels, reasons, or IPFS URIs.

## Changes Made

### 1. Stream Model (`stream_types.rs`)
- Created `StreamInfo` struct with optional metadata field
- Added `validate_metadata()` function with 512-character limit
- Metadata constraints: 1-512 characters when present, or None

### 2. Event Emission (`events.rs`)
- Added `emit_stream_created()` event function
- Event includes stream_id, creator, recipient, amount, and metadata presence flag
- Uses optimized event structure with `symbol_short!("strm_crt")`

### 3. Comprehensive Tests (`stream_metadata_test.rs`)
11 tests covering:
- ✅ Metadata present/absent scenarios
- ✅ Empty string validation (invalid)
- ✅ Maximum length (512 chars) - valid
- ✅ Exceeds max length (513 chars) - invalid
- ✅ Boundary testing (1 char, 511 chars)
- ✅ IPFS URI format
- ✅ Human-readable labels
- ✅ Event emission with/without metadata

## Validation Rules
- **Minimum length**: 1 character (when present)
- **Maximum length**: 512 characters
- **Empty strings**: Rejected with `Error::InvalidParameters`
- **None value**: Allowed (metadata is optional)

## Usage Examples

```rust
// With IPFS metadata
let stream = StreamInfo {
    id: 1,
    creator: creator_address,
    recipient: recipient_address,
    amount: 1000_0000000,
    metadata: Some(String::from_str(&env, "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG")),
    created_at: env.ledger().timestamp(),
};

// With label
let stream = StreamInfo {
    id: 2,
    creator: creator_address,
    recipient: recipient_address,
    amount: 5000_0000000,
    metadata: Some(String::from_str(&env, "Monthly salary payment")),
    created_at: env.ledger().timestamp(),
};

// Without metadata
let stream = StreamInfo {
    id: 3,
    creator: creator_address,
    recipient: recipient_address,
    amount: 100_0000000,
    metadata: None,
    created_at: env.ledger().timestamp(),
};
```

## Test Results
```
running 11 tests
test stream_metadata_test::test_metadata_boundary_1_char ... ok
test stream_metadata_test::test_metadata_boundary_511_chars ... ok
test stream_metadata_test::test_metadata_empty_string_invalid ... ok
test stream_metadata_test::test_metadata_exceeds_max_length ... ok
test stream_metadata_test::test_metadata_ipfs_uri ... ok
test stream_metadata_test::test_metadata_label ... ok
test stream_metadata_test::test_metadata_max_length_valid ... ok
test stream_metadata_test::test_stream_created_event_with_metadata ... ok
test stream_metadata_test::test_stream_created_event_without_metadata ... ok
test stream_metadata_test::test_stream_metadata_absent ... ok
test stream_metadata_test::test_stream_metadata_present ... ok

test result: ok. 11 passed; 0 failed
```

## Acceptance Criteria Met
- ✅ Optional metadata URI/string added to stream model
- ✅ Metadata length constraints validated (1-512 chars)
- ✅ Metadata presence emitted in stream-created event
- ✅ Tests for metadata present/absent scenarios
- ✅ Tests for invalid size limits (empty, too long)
- ✅ Boundary testing (1 char, 511 chars, 512 chars, 513 chars)
