# LESSONS - auto-maintained by scripts/lessons.py

> Machine-owned. Do NOT hand-edit. Changes are overwritten on the next `lessons.py` write.
> Canonical state lives in `.specs/lessons.json`. Edit lessons only via the script.
> promote_threshold=2 distinct features · window_days=45 · quarantine_threshold=2

## Confirmed (load these at Plan/Checks)

Corroborated across multiple features. Safe to apply as guidance.

_none_

## Candidates (under observation - do NOT load as guidance yet)

Seen once or not yet corroborated. Tracked, not trusted.

### L-001 - A documentation check that claims wording or ordering must grep for that wording, not only for the rule identifiers it cites
- signal: `spec_precision_gap` · recurrence: 1 feature(s) · scope: `readme` · harmful: 0
- features: wpc-53
- evidence: verification.md finding 1 - C8/C9 README proofs grep only for rule ids (readme)
- last seen: 2026-09-28T10:59:01Z

### L-002 - Scope a documentation grep to the smallest block that holds the claimed text, so text added nearby in the same change cannot satisfy it
- signal: `spec_precision_gap` · recurrence: 1 feature(s) · scope: `readme` · harmful: 0
- features: arcaboot-com-nome
- evidence: verification.md finding 1 - C20 sed range now spans the new naming section (README.md:394-396) (readme)
- last seen: 2026-09-28T12:32:54Z

### L-003 - A document block presented as a command's exact output must be pinned by a test that compares the whole text, or labelled as unpinned
- signal: `spec_precision_gap` · recurrence: 1 feature(s) · scope: `readme` · harmful: 0
- features: arcaboot-com-nome
- evidence: verification.md finding 2 - README.md:392 presented a refusal block as test-required text, test pins two substrings (src/dispositivo.rs:526-527) (readme)
- last seen: 2026-09-28T12:32:55Z

### L-004 - A documentation check that removes stale wording must also assert the wording that replaces it
- signal: `spec_precision_gap` · recurrence: 1 feature(s) · scope: `readme` · harmful: 0
- features: arcaboot-com-nome
- evidence: verification.md finding 3 - C17 proves the old C-10 wording is gone, not that README.md:812 and :1180 state the new one (readme)
- last seen: 2026-09-28T12:32:55Z

## Quarantined (failed when applied - ignore)

A confirmed lesson that recurred alongside failure. Kept for the maintainer to review.

_none_
