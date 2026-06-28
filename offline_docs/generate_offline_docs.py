"""Generate a self-contained Lullaby documentation HTML bundle from Markdown.

This is the initial generator path for Epic 1.5. It intentionally uses only the
Python standard library so the docs build can run in release and installer
verification environments without a package manager.
"""

from __future__ import annotations

import argparse
import html
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = REPO_ROOT / "target" / "offline_docs" / "index.html"

SOURCE_DOCS = [
    ("Project Overview", REPO_ROOT / "README.md"),
    ("Core Language Rules", REPO_ROOT / "documents" / "core_language_rules.md"),
    ("Alpha 1 Language Surface", REPO_ROOT / "documents" / "alpha1_language_surface.md"),
    ("Diagnostics Registry", REPO_ROOT / "documents" / "diagnostic_registry.md"),
    ("Release Notes", REPO_ROOT / "documents" / "alpha1_release_notes.md"),
    ("Post-Alpha Roadmap", REPO_ROOT / "documents" / "post_alpha_roadmap.md"),
]


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return slug or "section"


def inline_markdown(value: str) -> str:
    escaped = html.escape(value)
    escaped = re.sub(r"`([^`]+)`", r"<code>\1</code>", escaped)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    return escaped


def flush_list(output: list[str], list_items: list[str]) -> None:
    if not list_items:
        return
    output.append("<ul>")
    output.extend(f"<li>{item}</li>" for item in list_items)
    output.append("</ul>")
    list_items.clear()


def markdown_to_html(markdown: str) -> str:
    output: list[str] = []
    list_items: list[str] = []
    in_code = False
    code_lines: list[str] = []

    for raw_line in markdown.splitlines():
        line = raw_line.rstrip()

        if line.startswith("```"):
            if in_code:
                output.append(f"<pre><code>{html.escape(chr(10).join(code_lines))}</code></pre>")
                code_lines.clear()
                in_code = False
            else:
                flush_list(output, list_items)
                in_code = True
            continue

        if in_code:
            code_lines.append(line)
            continue

        if not line:
            flush_list(output, list_items)
            continue

        heading = re.match(r"^(#{1,4})\s+(.+)$", line)
        if heading:
            flush_list(output, list_items)
            level = min(len(heading.group(1)) + 1, 5)
            text = heading.group(2).strip()
            output.append(f'<h{level} id="{slugify(text)}">{inline_markdown(text)}</h{level}>')
            continue

        bullet = re.match(r"^[-*]\s+(.+)$", line)
        if bullet:
            list_items.append(inline_markdown(bullet.group(1)))
            continue

        table_like = line.startswith("|") and line.endswith("|")
        if table_like:
            flush_list(output, list_items)
            cells = [inline_markdown(cell.strip()) for cell in line.strip("|").split("|")]
            if all(set(cell.replace(":", "").replace("-", "")) == set() for cell in cells):
                continue
            output.append("<table><tr>" + "".join(f"<td>{cell}</td>" for cell in cells) + "</tr></table>")
            continue

        flush_list(output, list_items)
        output.append(f"<p>{inline_markdown(line)}</p>")

    flush_list(output, list_items)
    if in_code:
        output.append(f"<pre><code>{html.escape(chr(10).join(code_lines))}</code></pre>")

    return "\n".join(output)


def render_document() -> str:
    nav_items = []
    sections = []

    for title, source in SOURCE_DOCS:
        if not source.exists():
            raise FileNotFoundError(f"required documentation source not found: {source}")
        slug = slugify(title)
        nav_items.append(f'<li><a href="#{slug}">{html.escape(title)}</a></li>')
        body = markdown_to_html(source.read_text(encoding="utf-8"))
        sections.append(
            f'<section id="{slug}"><h1>{html.escape(title)}</h1>'
            f'<p class="source">Source: <code>{html.escape(source.relative_to(REPO_ROOT).as_posix())}</code></p>'
            f"{body}</section>"
        )

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Lullaby Generated Offline Documentation</title>
  <style>
    :root {{ color-scheme: light dark; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }}
    body {{ margin: 0; line-height: 1.55; }}
    header {{ padding: 2rem; background: #18212f; color: #f7fbff; }}
    main {{ display: grid; grid-template-columns: minmax(14rem, 18rem) minmax(0, 1fr); gap: 2rem; padding: 1.5rem; }}
    nav {{ position: sticky; top: 1rem; align-self: start; }}
    nav ul {{ list-style: none; padding: 0; }}
    nav li {{ margin: 0.4rem 0; }}
    a {{ color: #235fa7; }}
    section {{ max-width: 76rem; margin-bottom: 3rem; }}
    code, pre {{ font-family: "Cascadia Mono", Consolas, monospace; }}
    code {{ background: rgba(127, 127, 127, 0.14); padding: 0.08rem 0.25rem; border-radius: 0.2rem; }}
    pre {{ overflow-x: auto; padding: 1rem; background: #111827; color: #f9fafb; }}
    table {{ border-collapse: collapse; margin: 0.75rem 0; width: 100%; }}
    td {{ border: 1px solid rgba(127, 127, 127, 0.35); padding: 0.4rem; vertical-align: top; }}
    .source {{ color: #697386; }}
    @media (max-width: 760px) {{ main {{ display: block; }} nav {{ position: static; }} }}
  </style>
</head>
<body>
  <header>
    <h1>Lullaby Generated Offline Documentation</h1>
    <p>Self-contained HTML generated from canonical repository Markdown.</p>
  </header>
  <main>
    <nav aria-label="Documentation sections"><ul>{''.join(nav_items)}</ul></nav>
    <div>{''.join(sections)}</div>
  </main>
</body>
</html>
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "output",
        nargs="?",
        type=Path,
        default=DEFAULT_OUTPUT,
        help=f"output HTML path, default: {DEFAULT_OUTPUT}",
    )
    args = parser.parse_args()

    output = args.output
    if not output.is_absolute():
        output = REPO_ROOT / output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(render_document(), encoding="utf-8", newline="\n")
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
