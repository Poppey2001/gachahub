#!/usr/bin/env python3
"""GachaHub i18n audit.

Checks the complete frontend source tree for:
- translation keys used via t('...') / translate(..., '...')
- keys missing from the dictionaries
- supported languages that only inherit/fallback to another language
- placeholder mismatches ({{name}})
- duplicate dictionary keys
- unused translation keys
- likely hard-coded user-facing JSX text/attributes (heuristic)

No third-party Python packages are required.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

PROJECT_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_I18N = PROJECT_ROOT / "src" / "i18n" / "index.ts"
DEFAULT_SOURCE = PROJECT_ROOT / "src"
LANGUAGES = ("de", "en", "fr", "es")
BASE_LANGUAGE = "en"
SOURCE_EXTENSIONS = {".ts", ".tsx", ".js", ".jsx"}
SKIP_DIRS = {"node_modules", "dist", "target", ".git", ".vite"}

KEY_VALUE_RE = re.compile(
    r"(?P<q>['\"])(?P<key>(?:\\.|(?!\1).)+?)(?P=q)\s*:\s*"
    r"(?P<vq>['\"])(?P<value>(?:\\.|(?!\3).)*?)(?P=vq)",
    re.DOTALL,
)
STATIC_T_RE = re.compile(r"\bt\s*\(\s*(?P<q>['\"`])(?P<key>[^'\"`]+)(?P=q)")
STATIC_TRANSLATE_RE = re.compile(
    r"\btranslate\s*\([^,]+,\s*(?P<q>['\"`])(?P<key>[^'\"`]+)(?P=q)"
)
ANY_T_CALL_RE = re.compile(r"\bt\s*\(\s*([^\s'\"`][^,)]*)")
PLACEHOLDER_RE = re.compile(r"\{\{\s*([A-Za-z_][A-Za-z0-9_]*)\s*\}\}")
JSX_TEXT_RE = re.compile(r">([^<>{}\n]+)<")
JSX_ATTR_RE = re.compile(
    r"\b(?:title|placeholder|aria-label|aria-description)\s*=\s*(['\"])(.*?)\1",
    re.DOTALL,
)
WORD_RE = re.compile(r"[A-Za-zÀ-ÖØ-öø-ÿ]", re.UNICODE)

# Strings that are generally identifiers/technical labels rather than UI copy.
HARDCODE_ALLOW = {
    "GachaHub", "XXMI", "GIMI", "SRMI", "ZZMI", "WWMI", "EFMI", "HIMI",
    "Wine", "Proton", "UMU", "GameMode", "Gamescope", "MangoHud", "Auto",
    "Windows", "Linux", "Steam", "Heroic", "Lutris", "Bottles", "FPS",
}


@dataclass(frozen=True)
class Location:
    path: str
    line: int


@dataclass
class LanguageData:
    explicit: dict[str, str]
    parents: list[str]
    duplicates: dict[str, list[int]]


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def iter_source_files(root: Path) -> Iterable[Path]:
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix not in SOURCE_EXTENSIONS:
            continue
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        yield path


def find_matching_brace(text: str, open_pos: int) -> int:
    depth = 0
    quote: str | None = None
    escaped = False
    line_comment = False
    block_comment = False
    i = open_pos
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if line_comment:
            if ch == "\n":
                line_comment = False
            i += 1
            continue
        if block_comment:
            if ch == "*" and nxt == "/":
                block_comment = False
                i += 2
                continue
            i += 1
            continue
        if quote:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            i += 1
            continue
        if ch == "/" and nxt == "/":
            line_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "*":
            block_comment = True
            i += 2
            continue
        if ch in "'\"`":
            quote = ch
            i += 1
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError(f"Unmatched '{{' at byte {open_pos}")


def decode_ts_string(value: str) -> str:
    # We only need human-readable values/placeholders. Preserve unknown escapes.
    return (
        value.replace(r"\'", "'")
        .replace(r'\"', '"')
        .replace(r"\\", "\\")
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
    )


def parse_dictionary_body(body: str, body_start: int, whole_text: str):
    entries: list[tuple[str, str, int]] = []
    for match in KEY_VALUE_RE.finditer(body):
        key = decode_ts_string(match.group("key"))
        value = decode_ts_string(match.group("value"))
        entries.append((key, value, line_number(whole_text, body_start + match.start())))
    parents = re.findall(r"\.\.\.\s*([A-Za-z_$][\w$]*)", body)
    return entries, parents


def parse_i18n(path: Path) -> dict[str, LanguageData]:
    text = path.read_text(encoding="utf-8")
    raw_entries: dict[str, list[tuple[str, str, int]]] = defaultdict(list)
    parents: dict[str, list[str]] = defaultdict(list)

    for lang in LANGUAGES:
        start_re = re.compile(rf"\bconst\s+{re.escape(lang)}\s*:\s*Dictionary\s*=\s*\{{")
        m = start_re.search(text)
        if not m:
            continue
        open_pos = text.find("{", m.start())
        close_pos = find_matching_brace(text, open_pos)
        body_start = open_pos + 1
        body = text[body_start:close_pos]
        entries, inherited = parse_dictionary_body(body, body_start, text)
        raw_entries[lang].extend(entries)
        parents[lang].extend(p for p in inherited if p in LANGUAGES)

    assign_re = re.compile(r"Object\.assign\(\s*(de|en|fr|es)\s*,\s*\{")
    for m in assign_re.finditer(text):
        lang = m.group(1)
        open_pos = text.find("{", m.start())
        close_pos = find_matching_brace(text, open_pos)
        body_start = open_pos + 1
        body = text[body_start:close_pos]
        entries, inherited = parse_dictionary_body(body, body_start, text)
        raw_entries[lang].extend(entries)
        parents[lang].extend(p for p in inherited if p in LANGUAGES)

    result: dict[str, LanguageData] = {}
    for lang in LANGUAGES:
        explicit: dict[str, str] = {}
        lines: dict[str, list[int]] = defaultdict(list)
        for key, value, line in raw_entries.get(lang, []):
            explicit[key] = value
            lines[key].append(line)
        result[lang] = LanguageData(
            explicit=explicit,
            parents=list(dict.fromkeys(parents.get(lang, []))),
            duplicates={key: ls for key, ls in lines.items() if len(ls) > 1},
        )
    return result


def effective_dictionary(lang: str, data: dict[str, LanguageData], trail: tuple[str, ...] = ()) -> dict[str, str]:
    if lang in trail:
        return dict(data[lang].explicit)
    merged: dict[str, str] = {}
    for parent in data[lang].parents:
        merged.update(effective_dictionary(parent, data, trail + (lang,)))
    merged.update(data[lang].explicit)
    return merged


def scan_usage(source_root: Path, i18n_path: Path):
    used: dict[str, list[Location]] = defaultdict(list)
    dynamic: list[tuple[Location, str]] = []
    hardcoded: list[tuple[Location, str, str]] = []

    for path in iter_source_files(source_root):
        if path.resolve() == i18n_path.resolve():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        rel = str(path.relative_to(PROJECT_ROOT))

        occupied: list[tuple[int, int]] = []
        for regex in (STATIC_T_RE, STATIC_TRANSLATE_RE):
            for match in regex.finditer(text):
                key = match.group("key")
                used[key].append(Location(rel, line_number(text, match.start())))
                occupied.append(match.span())

        for match in ANY_T_CALL_RE.finditer(text):
            if any(a <= match.start() < b for a, b in occupied):
                continue
            expr = " ".join(match.group(1).strip().split())[:100]
            dynamic.append((Location(rel, line_number(text, match.start())), expr))

        if path.suffix not in {".tsx", ".jsx"}:
            continue

        for match in JSX_TEXT_RE.finditer(text):
            candidate = " ".join(match.group(1).split())
            if not candidate or not WORD_RE.search(candidate):
                continue
            if candidate in HARDCODE_ALLOW or candidate.startswith("//"):
                continue
            if any(token in candidate for token in ("===", "!==", "=>", "&&", "||", " ? ", " : ")):
                continue
            if "i18n-ignore" in text[max(0, match.start() - 120):match.end() + 20]:
                continue
            # Skip pure technical tokens/identifiers and very short labels.
            if len(candidate) < 3 or re.fullmatch(r"[A-Za-z0-9_.:/+@-]+", candidate) and " " not in candidate:
                continue
            hardcoded.append((Location(rel, line_number(text, match.start())), "text", candidate[:180]))

        for match in JSX_ATTR_RE.finditer(text):
            candidate = " ".join(match.group(2).split())
            if not candidate or not WORD_RE.search(candidate) or candidate in HARDCODE_ALLOW:
                continue
            if "i18n-ignore" in text[max(0, match.start() - 120):match.end() + 20]:
                continue
            hardcoded.append((Location(rel, line_number(text, match.start())), "attribute", candidate[:180]))

    return used, dynamic, hardcoded


def placeholders(value: str) -> set[str]:
    return set(PLACEHOLDER_RE.findall(value))


def make_report(data, used, dynamic, hardcoded):
    effective = {lang: effective_dictionary(lang, data) for lang in LANGUAGES}
    all_keys = set().union(*(set(d.explicit) for d in data.values()))
    used_keys = set(used)

    missing_effective = {lang: sorted(all_keys - set(effective[lang])) for lang in LANGUAGES}
    missing_explicit = {lang: sorted(all_keys - set(data[lang].explicit)) for lang in LANGUAGES}
    inherited = {
        lang: sorted((set(effective[lang]) - set(data[lang].explicit)) & all_keys)
        for lang in LANGUAGES
    }
    undefined_used = sorted(used_keys - all_keys)
    unused = sorted(all_keys - used_keys)

    base_effective = effective[BASE_LANGUAGE]
    placeholder_mismatches = []
    identical_to_base = []
    for lang in LANGUAGES:
        if lang == BASE_LANGUAGE:
            continue
        for key in sorted(set(base_effective) & set(effective[lang])):
            base_vars = placeholders(base_effective[key])
            lang_vars = placeholders(effective[lang][key])
            if base_vars != lang_vars:
                placeholder_mismatches.append({
                    "language": lang,
                    "key": key,
                    "base": sorted(base_vars),
                    "translation": sorted(lang_vars),
                })
            if key in data[lang].explicit and effective[lang][key].strip() == base_effective[key].strip():
                identical_to_base.append({"language": lang, "key": key, "value": effective[lang][key]})

    duplicates = []
    for lang in LANGUAGES:
        for key, lines in sorted(data[lang].duplicates.items()):
            duplicates.append({"language": lang, "key": key, "lines": lines})

    return {
        "languages": list(LANGUAGES),
        "base_language": BASE_LANGUAGE,
        "counts": {
            "dictionary_keys": len(all_keys),
            "used_static_keys": len(used_keys),
            "unused_keys": len(unused),
            "dynamic_calls": len(dynamic),
            "hardcoded_candidates": len(hardcoded),
        },
        "language_counts": {
            lang: {
                "explicit": len(data[lang].explicit),
                "effective": len(effective[lang]),
                "inherited": len(inherited[lang]),
                "missing_effective": len(missing_effective[lang]),
                "missing_explicit": len(missing_explicit[lang]),
                "parents": data[lang].parents,
            }
            for lang in LANGUAGES
        },
        "undefined_used_keys": [
            {"key": key, "locations": [loc.__dict__ for loc in used[key]]}
            for key in undefined_used
        ],
        "missing_effective": missing_effective,
        "missing_explicit": missing_explicit,
        "inherited_fallbacks": inherited,
        "placeholder_mismatches": placeholder_mismatches,
        "duplicates": duplicates,
        "unused_keys": unused,
        "dynamic_calls": [
            {"location": loc.__dict__, "expression": expr} for loc, expr in dynamic
        ],
        "hardcoded_candidates": [
            {"location": loc.__dict__, "kind": kind, "text": txt}
            for loc, kind, txt in hardcoded
        ],
        "identical_to_base": identical_to_base,
    }


def print_group(title: str, items, limit: int = 80):
    if not items:
        return
    print(f"\n{title} ({len(items)})")
    print("-" * min(88, len(title) + 8))
    for item in items[:limit]:
        print(item)
    if len(items) > limit:
        print(f"... +{len(items) - limit} weitere")


def print_report(report, show_hardcoded: bool):
    print("GachaHub i18n audit")
    print("===================")
    c = report["counts"]
    print(f"Dictionary keys : {c['dictionary_keys']}")
    print(f"Used static keys: {c['used_static_keys']}")
    print(f"Unused keys     : {c['unused_keys']}")
    print(f"Dynamic t(...)  : {c['dynamic_calls']}")
    if show_hardcoded:
        print(f"Hardcoded hints : {c['hardcoded_candidates']}")

    print("\nLanguages")
    print("---------")
    for lang in LANGUAGES:
        lc = report["language_counts"][lang]
        parents = f"; fallback from {', '.join(lc['parents'])}" if lc["parents"] else ""
        print(
            f"{lang}: explicit {lc['explicit']}, effective {lc['effective']}, "
            f"inherited {lc['inherited']}, missing {lc['missing_effective']}{parents}"
        )

    print_group(
        "ERROR: used keys not defined in any dictionary",
        [f"{x['key']}  ->  " + ", ".join(f"{l['path']}:{l['line']}" for l in x["locations"]) for x in report["undefined_used_keys"]],
    )
    for lang in LANGUAGES:
        print_group(f"ERROR: {lang} has no effective value", report["missing_effective"][lang])
    print_group(
        "ERROR: placeholder mismatches",
        [f"{x['language']} {x['key']}: base={x['base']} translation={x['translation']}" for x in report["placeholder_mismatches"]],
    )
    print_group(
        "WARNING: duplicate dictionary keys",
        [f"{x['language']} {x['key']} at lines {', '.join(map(str, x['lines']))}" for x in report["duplicates"]],
    )
    for lang in LANGUAGES:
        if report["inherited_fallbacks"][lang]:
            print_group(
                f"MISSING TRANSLATION: {lang} currently falls back/inherits",
                report["inherited_fallbacks"][lang],
            )
    print_group(
        "WARNING: explicit translation identical to English",
        [f"{x['language']} {x['key']} = {x['value']}" for x in report["identical_to_base"]],
    )
    print_group("INFO: unused dictionary keys", report["unused_keys"])
    print_group(
        "INFO: dynamic t(...) calls require manual review",
        [f"{x['location']['path']}:{x['location']['line']} -> {x['expression']}" for x in report["dynamic_calls"]],
    )
    if show_hardcoded:
        print_group(
            "POSSIBLE HARDCODED UI TEXT (manual review)",
            [f"{x['location']['path']}:{x['location']['line']} [{x['kind']}] {x['text']}" for x in report["hardcoded_candidates"]],
            limit=120,
        )


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit GachaHub translations across the frontend project.")
    parser.add_argument("--i18n", type=Path, default=DEFAULT_I18N, help="Path to src/i18n/index.ts")
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE, help="Frontend source root")
    parser.add_argument("--json", type=Path, help="Also write a machine-readable JSON report")
    parser.add_argument("--strict", action="store_true", help="Fail if any supported language relies on fallback/inheritance")
    parser.add_argument("--hardcoded", action="store_true", help="Report likely hard-coded user-facing JSX strings")
    parser.add_argument("--fail-hardcoded", action="store_true", help="Fail if hard-coded UI candidates are found (implies --hardcoded)")
    args = parser.parse_args()
    if args.fail_hardcoded:
        args.hardcoded = True

    if not args.i18n.exists():
        print(f"ERROR: i18n file not found: {args.i18n}", file=sys.stderr)
        return 2
    if not args.source.exists():
        print(f"ERROR: source directory not found: {args.source}", file=sys.stderr)
        return 2

    data = parse_i18n(args.i18n)
    missing_langs = [lang for lang in LANGUAGES if not data[lang].explicit and not data[lang].parents]
    if missing_langs:
        print(f"ERROR: could not parse dictionaries: {', '.join(missing_langs)}", file=sys.stderr)
        return 2

    used, dynamic, hardcoded = scan_usage(args.source, args.i18n)
    report = make_report(data, used, dynamic, hardcoded)
    print_report(report, args.hardcoded)

    if args.json:
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"\nJSON report: {args.json}")

    hard_errors = (
        len(report["undefined_used_keys"])
        + sum(len(v) for v in report["missing_effective"].values())
        + len(report["placeholder_mismatches"])
    )
    strict_errors = sum(len(report["inherited_fallbacks"][lang]) for lang in LANGUAGES) if args.strict else 0
    hardcoded_errors = len(report["hardcoded_candidates"]) if args.fail_hardcoded else 0

    if hard_errors or strict_errors or hardcoded_errors:
        print(
            f"\nRESULT: FAIL (hard={hard_errors}, strict={strict_errors}, hardcoded={hardcoded_errors})",
            file=sys.stderr,
        )
        return 1

    print("\nRESULT: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
