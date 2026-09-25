# GachaHub v0.14.3

## Translation audit

- Added `scripts/check-i18n.py`.
- Added strict missing-translation checks for German, English, French and Spanish.
- Detects fallback-only/inherited translations, missing keys, placeholder mismatches, duplicates and unused keys.
- Scans the whole frontend source tree for translation-key usage.
- Optional heuristic scan for hard-coded TSX/JSX UI strings.
- Added `npm run i18n:check` and `npm run i18n:audit`.
- JSON report output supported for CI and editor tooling.
