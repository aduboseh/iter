# Iter External Spec v1

Iter’s governance surface is frozen by the JSON Schemas under `schemas/v1`. Every request/response MUST validate against these artifacts.

## Schemas

| Contract | Schema |
| --- | --- |
| DecisionPacket (authoritative response) | `schemas/v1/decision_packet.schema.json` |
| DecisionPreview (simulation response) | `schemas/v1/decision_preview.schema.json` |
| decision.check request payload | `schemas/v1/decision_check_request.schema.json` |
| audit.search filter/result | `schemas/v1/audit_search.schema.json` (definitions `AuditSearchFilter`, `AuditSearchResult`) |

Each schema is Draft 7, `additionalProperties: false`, and versioned via `$id = https://iter.dev/schemas/v1/...`.

## Integration Guidance

### decision.check
1. Construct a payload that matches `decision_check_request.schema.json`.
2. Validate locally (any JSON Schema validator).
3. Send via MCP `decision.check`.
4. Responses conform to `decision_packet.schema.json`.

### decision.preview
1. Request matches `decision_check_request.schema.json`.
2. Response matches `decision_preview.schema.json`.

### audit.search
1. Request filter matches the `#/definitions/AuditSearchFilter` definition inside `audit_search.schema.json`.
2. Response matches `#/definitions/AuditSearchResult`.

## Claim Gate Tests

`cargo test` enforces two invariants:

1. `tests/schema_integrity.rs` serializes canonical Rust structs (and the golden DecisionPacket fixture) then validates them against the committed schemas.
2. `tests/doc_examples_integrity.rs` validates `tests/data/golden_decision_v1.json`, ensuring documentation examples stay in sync. Full markdown extraction will be added once tagged JSON blocks exist.

Any change to the structs that would affect the schemas must regenerate artifacts before tests pass.

## Regenerating Schemas

1. Ensure changes are intentional and backwards compatible (or bump the schema version).
2. Run:
   ```
   cargo run --features schema-gen --bin generate-schemas
   ```
3. Review the updated files in `schemas/v1`.
4. Commit the regenerated artifacts alongside the code change.

`schema-gen` is an opt-in feature gate; it has **no effect** on default builds or runtime behavior.
