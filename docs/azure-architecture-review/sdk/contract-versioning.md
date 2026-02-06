# SDK Contract Versioning

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Version Semantics

**Format:** MAJOR.MINOR.PATCH

### PATCH (x.y.Z)

- Bug fixes
- Documentation updates
- No API changes

**Breaking:** No

### MINOR (x.Y.0)

- Add new tools
- Add optional parameters
- Add new enum values

**Breaking:** Potentially (new enums break old readers)

### MAJOR (X.0.0)

- Remove tools
- Change tool contracts
- Change required parameters

**Breaking:** Yes

---

## Breaking Change Definition

**A change is breaking if:**
- Old SDK cannot communicate with new server
- New SDK cannot communicate with old server
- Checksum compatibility is violated

---

## Checksum Compatibility

**Guarantee:** Same contract version → identical checksums for identical inputs.

**Violation:** Checksum mismatch across SDK versions with same contract version.

**Enforcement:** Cross-version replay tests in CI.

---

## Deprecation Policy

**Deprecated features:**
- Marked as deprecated in docs
- Remain functional for 2 minor versions
- Removed in next major version

**Example:**
- Deprecated in 1.5.0
- Functional through 1.7.x
- Removed in 2.0.0
