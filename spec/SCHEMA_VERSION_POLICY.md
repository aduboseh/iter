# Schema Version Policy

This document defines version management for Iter MCP protocol schemas.

## Current Version

**Schema Version: 1.0.0** (aligned with protocol version)

## Schema Location

All schemas are located in the `spec/` directory and use GitHub blob URLs as canonical `$id` references:

```
https://github.com/aduboseh/iter/blob/v1.0.0/spec/<schema>.json
```

This ensures:
- Schemas are immutable per release tag
- No external domain dependencies
- Verifiable via git history

## Versioning Rules

1. **Schema version tracks protocol version** - Schema changes require protocol version bump
2. **Breaking changes require major bump** - Field removal, type changes, required field additions
3. **Additive changes require minor bump** - New optional fields, new enum values
4. **Documentation changes require patch bump** - Description updates, examples

## Compatibility

| Change Type | Version Impact | Backward Compatible |
|-------------|----------------|---------------------|
| Add optional field | Minor | Yes |
| Add required field | Major | No |
| Remove field | Major | No |
| Change field type | Major | No |
| Add enum value | Minor | Yes |
| Remove enum value | Major | No |

## Validation

Schemas use JSON Schema Draft 2020-12. Validate with:

```bash
# Using ajv-cli
npx ajv validate -s spec/mcp_node_state.schema.json -d response.json
```

## History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2024-12-15 | Initial stable release, aligned with Iter v1.0.0 |
| 0.3.0 | 2024-12-01 | Pre-release schemas (superseded) |
