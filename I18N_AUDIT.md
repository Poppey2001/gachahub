# GachaHub i18n audit

GachaHub includes `scripts/check-i18n.py`, a dependency-free project-wide translation audit.

## Quick checks

```bash
npm run i18n:check
```

Strict CI-style check. It exits with status 1 when:

- a `t('...')` key is not defined;
- a supported language has no effective value for a key;
- `{{variable}}` placeholders do not match the English source string;
- a supported language is only using a fallback/inherited value instead of an explicit translation.

For a broader manual audit:

```bash
npm run i18n:audit
```

This additionally scans TSX/JSX for likely hard-coded UI text and writes `i18n-report.json`.
Hard-coded text detection is intentionally heuristic, so its results should be reviewed rather than blindly treated as errors.

## Direct usage

```bash
python3 scripts/check-i18n.py
python3 scripts/check-i18n.py --strict
python3 scripts/check-i18n.py --hardcoded
python3 scripts/check-i18n.py --hardcoded --json i18n-report.json
python3 scripts/check-i18n.py --fail-hardcoded
```

## What is scanned

- `src/**/*.{ts,tsx,js,jsx}`
- static `t('translation.key')` calls
- static `translate(language, 'translation.key')` calls
- the `de`, `en`, `fr`, and `es` dictionaries in `src/i18n/index.ts`
- dictionary additions through `Object.assign(language, {...})`
- fallback/inheritance such as `...en`
- interpolation variables such as `{{value}}`
- duplicate and unused keys
- likely hard-coded JSX text and common visible attributes (`title`, `placeholder`, `aria-label`, `aria-description`)

Technical/provider strings may intentionally remain untranslated. For intentional JSX exceptions, place `i18n-ignore` close to the text so the hard-coded-text heuristic ignores it.
