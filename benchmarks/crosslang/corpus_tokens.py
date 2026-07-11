#!/usr/bin/env python3
"""Tokenize the whole cross-language corpus and emit JSON for the benchmark
artifact. Walks benchmarks/crosslang/corpus/<category>/{lullaby.lby,c.c,cpp.cpp,
rust.rs,python.py} (plus the legacy scalar suite as category 'numeric_basics'),
extracts each function definition per language, strips comments/boilerplate/the
verification driver, counts o200k_base tokens, and matches functions across
languages by name.

Output:
  - benchmarks/crosslang/corpus_data.json  (per-function tokens + source, totals)
  - a printed summary table

Run with a tiktoken-capable Python:
  C:/Users/emil/AppData/Local/Programs/Python/Python314/python.exe benchmarks/crosslang/corpus_tokens.py
"""
import json
import re
from pathlib import Path

import tiktoken

ENC = tiktoken.get_encoding("o200k_base")
ROOT = Path(__file__).resolve().parent
CORPUS = ROOT / "corpus"

LANGS = ["Lullaby", "C", "C++", "Rust", "Python"]
EXT = {"Lullaby": "lullaby.lby", "C": "c.c", "C++": "cpp.cpp", "Rust": "rust.rs", "Python": "python.py"}
KIND = {"Lullaby": "lullaby", "C": "clike", "C++": "clike", "Rust": "rust", "Python": "python"}
DRIVER = {"Lullaby": "fn main", "C": "int main", "C++": "int main", "Rust": "fn main", "Python": "if __name__"}


def strip_comments(text, kind):
    if kind in ("clike", "rust"):
        text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
        text = re.sub(r"//[^\n]*", "", text)
    else:
        text = re.sub(r"#[^\n]*", "", text)
    return text


def strip_boilerplate(text, kind):
    out = []
    for ln in text.splitlines():
        s = ln.strip()
        if kind in ("clike", "rust") and (s.startswith("#") or s.startswith("using ") or s.startswith("use ")):
            continue
        if kind == "python" and (s.startswith("import ") or s.startswith("from ")):
            continue
        out.append(ln)
    return "\n".join(out)


def before_driver(text, marker):
    i = text.find(marker)
    return text[:i] if i >= 0 else text


def split_clike(region):
    funcs, depth, start = [], 0, 0
    for i, ch in enumerate(region):
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                funcs.append(region[start:i + 1].strip())
                start = i + 1
    return [f for f in funcs if f and "(" in f.split("{")[0]]


def split_prefix(region, kw):
    funcs, cur, started = [], [], False
    for ln in region.splitlines():
        top = ln[:1] not in (" ", "\t")
        if top and ln.lstrip().startswith(kw + " "):
            if cur:
                funcs.append("\n".join(cur))
            cur = [ln]
            started = True
        elif started:
            cur.append(ln)
    if cur:
        funcs.append("\n".join(cur))
    return [f.strip() for f in funcs if f.strip()]


def func_name(chunk, kind):
    if kind == "rust" or kind == "lullaby":
        m = re.search(r"\bfn\s+(\w+)", chunk)
        return m.group(1) if m else None
    if kind == "python":
        m = re.search(r"\bdef\s+(\w+)", chunk)
        return m.group(1) if m else None
    sig = chunk.split("(")[0]
    ids = re.findall(r"[A-Za-z_]\w*", sig)
    return ids[-1] if ids else None


def tok(s):
    return len(ENC.encode(s.strip()))


def extract(path, kind, marker):
    """Return {func_name: (tokens, source)} for one language file."""
    if not path.exists():
        return {}
    raw = path.read_text(encoding="utf-8", errors="replace")
    region = strip_boilerplate(strip_comments(before_driver(raw, marker), kind), kind)
    region = re.sub(r"\n\s*\n+", "\n", region).strip()
    if kind in ("clike", "rust"):
        chunks = split_clike(region)
    elif kind == "python":
        chunks = split_prefix(region, "def")
    else:
        chunks = split_prefix(region, "fn")
    out = {}
    for ch in chunks:
        n = func_name(ch, kind)
        if n and n not in ("main",):
            out[n] = (tok(ch), ch)
    return out


# Discover categories: corpus/<cat>/ plus the legacy scalar suite.
categories = {}
if CORPUS.exists():
    for d in sorted(CORPUS.iterdir()):
        if d.is_dir():
            categories[d.name] = {lang: d / EXT[lang] for lang in LANGS}
# legacy scalar suite -> numeric_basics
scalar = {lang: ROOT / {"Lullaby": "lullaby/scalar.lby", "C": "c/scalar.c", "C++": "cpp/scalar.cpp",
                        "Rust": "rust/scalar.rs", "Python": "python/scalar.py"}[lang] for lang in LANGS}
if all(p.exists() for p in scalar.values()):
    categories["numeric_basics"] = scalar

funcs_out = []
totals = {lang: 0 for lang in LANGS}
cat_totals = {}
for cat, paths in categories.items():
    per_lang = {lang: extract(paths[lang], KIND[lang], DRIVER[lang]) for lang in LANGS}
    names = sorted(set().union(*[set(d.keys()) for d in per_lang.values()]))
    cat_totals[cat] = {lang: 0 for lang in LANGS}
    for name in names:
        entry = {"name": name, "category": cat, "langs": {}}
        present = 0
        for lang in LANGS:
            if name in per_lang[lang]:
                t, src = per_lang[lang][name]
                entry["langs"][lang] = {"tokens": t, "source": src}
                present += 1
        # only count toward totals functions present in Lullaby + >=1 other (real comparisons)
        if "Lullaby" in entry["langs"] and present >= 2:
            for lang, v in entry["langs"].items():
                totals[lang] += v["tokens"]
                cat_totals[cat][lang] += v["tokens"]
        entry["present"] = present
        funcs_out.append(entry)

data = {
    "langs": LANGS,
    "functions": funcs_out,
    "totals": totals,
    "categories": cat_totals,
    "n_functions": len([f for f in funcs_out if "Lullaby" in f["langs"] and f["present"] >= 2]),
}
(ROOT / "corpus_data.json").write_text(json.dumps(data, indent=1), encoding="utf-8")

# summary
print(f"corpus: {data['n_functions']} comparable functions across {len(categories)} categories\n")
print(f"{'category':<20}" + "".join(f"{l:>9}" for l in LANGS))
print("-" * (20 + 9 * len(LANGS)))
for cat in sorted(cat_totals):
    print(f"{cat:<20}" + "".join(f"{cat_totals[cat][l]:>9}" for l in LANGS))
print("-" * (20 + 9 * len(LANGS)))
print(f"{'TOTAL tokens':<20}" + "".join(f"{totals[l]:>9}" for l in LANGS))
base = totals["Lullaby"] or 1
print("\nvs Lullaby:")
for l in LANGS:
    print(f"  {l:<8} {totals[l]:>6}  ({totals[l]/base:.2f}x)")
print(f"\nwrote {ROOT/'corpus_data.json'}")
