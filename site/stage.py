#!/usr/bin/env python3
"""Copy the pages listed in SUMMARY.md into target/site-src for guidebook, pointing links to files left out at GitHub."""

import pathlib
import posixpath
import re
import shutil

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "target" / "site-src"
BLOB = "https://github.com/lu-zero/winnow-args/blob/master/"
TREE = "https://github.com/lu-zero/winnow-args/tree/master/"
LINK = re.compile(r"(\]\()([^)\s]+)(\))")


def rewrite(text: str, page: str, staged: set[str], images: dict[str, str]) -> str:
    here = posixpath.dirname(page)

    def fix(m: re.Match) -> str:
        target = m.group(2)
        if re.match(r"^[a-z][a-z0-9+.-]*:|^#|^/", target):
            return m.group(0)
        path, _, anchor = target.partition("#")
        resolved = posixpath.normpath(posixpath.join(here, path))
        if resolved in images:
            path = posixpath.relpath(images[resolved], here)
        elif resolved in staged:
            return m.group(0)
        else:
            base = TREE if (ROOT / resolved).is_dir() else BLOB
            return f"{m.group(1)}{base}{resolved}{'#' + anchor if anchor else ''}{m.group(3)}"
        return f"{m.group(1)}{path}{'#' + anchor if anchor else ''}{m.group(3)}"

    return LINK.sub(fix, text)


def main() -> None:
    summary = (ROOT / "site" / "SUMMARY.md").read_text(encoding="utf-8")
    pages = re.findall(r"\]\(([^)]+\.md)\)", summary)
    figures = sorted((ROOT / "docs").glob("*.svg"))
    images = {p.relative_to(ROOT).as_posix(): f"docs/images/{p.name}" for p in figures}
    sources = {dest: src for src, dest in images.items()}
    # guidebook resolves inlined SVG paths from the output root, not the page's directory
    sources.update({f"images/{p.name}": p.relative_to(ROOT).as_posix() for p in figures})
    staged = set(pages) | set(sources)
    shutil.rmtree(OUT, ignore_errors=True)
    for rel in sorted(staged):
        dest = OUT / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        if rel.endswith(".md"):
            text = (ROOT / rel).read_text(encoding="utf-8")
            dest.write_text(rewrite(text, rel, staged, images), encoding="utf-8")
        else:
            shutil.copyfile(ROOT / sources[rel], dest)
    (OUT / "SUMMARY.md").write_text(summary, encoding="utf-8")
    shutil.copyfile(ROOT / "site" / "book.json", OUT / "book.json")
    shutil.copytree(ROOT / "site" / "styles", OUT / "styles", dirs_exist_ok=True)


if __name__ == "__main__":
    main()
