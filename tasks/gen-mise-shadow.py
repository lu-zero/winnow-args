#!/usr/bin/env python3
"""Translate usage's mise shadow into winnow-args' vocabulary.

Reads bench/shadows/mise/src/lib.rs (usage's shadow, vendored; usage-derive)
and writes bench/shadows/mise-wa/src/lib.rs (winnow-args derive). Every `#[usage(...)]` item is
mapped or dropped by name; an unknown one stops the script. Every struct is
`unknown_flags = "value"`, usage's default. Selectors that name
no field of their own struct (an ancestor's global flag, which usage resolves at
runtime) are dropped and counted, since winnow-args resolves them at compile time.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "bench/shadows/mise/src/lib.rs"
TARGET = ROOT / "bench/shadows/mise-wa/src/lib.rs"

# Metadata winnow-args does not model; none of it changes how a line parses.
DROP = {"effect", "author", "bin", "mount", "value_enum", "var", "hide_default_value", "hide_env"}
KEEP = {
    "long", "short", "alias", "alias_hidden", "value_name", "help", "long_help", "after_long_help",
    "help_heading", "hide", "global", "count", "subcommand", "default", "env", "delimiter", "choices",
    "double_dash", "required", "default_missing", "restart_token", "group",
    "disable_help_flag", "disable_version_flag", "default_subcommand", "about", "long_about",
    "conflicts", "overrides", "requires", "required_unless",
}
SELECTORS = {"conflicts", "overrides", "requires", "required_unless"}


def split_top(text):
    """Split an attribute body on top-level commas, respecting strings, chars and parens."""
    items, depth, cur, i = [], 0, "", 0
    while i < len(text):
        c = text[i]
        if c == '"':
            j = i + 1
            while text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            cur += text[i : j + 1]
            i = j + 1
            continue
        if c == "'" and i + 2 < len(text) and (text[i + 2] == "'" or text[i + 1] == "\\"):
            j = text.index("'", i + 2 if text[i + 1] != "\\" else i + 3)
            cur += text[i : j + 1]
            i = j + 1
            continue
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        if c == "," and depth == 0:
            items.append(cur.strip())
            cur = ""
        else:
            cur += c
        i += 1
    if cur.strip():
        items.append(cur.strip())
    return items


def key_of(item):
    return re.split(r"[\s=(]", item, maxsplit=1)[0]


def strings_in(item):
    return re.findall(r'"((?:[^"\\]|\\.)*)"', item)


def attributes(text):
    """Yield (start, end, body) for every `#[usage(...)]`."""
    for m in re.finditer(r"#\[usage\(", text):
        i, depth = m.end(), 1
        while depth:
            c = text[i]
            if c == '"':
                i += 1
                while text[i] != '"':
                    i += 2 if text[i] == "\\" else 1
            elif c == "(":
                depth += 1
            elif c == ")":
                depth -= 1
            i += 1
        assert text[i] == "]", text[m.start() : i + 1]
        yield m.start(), i + 1, text[m.end() : i - 1]


def owners(text):
    """Map each struct's span to the selectors its fields answer to."""
    names = {}
    for m in re.finditer(r"pub struct (\w+) \{(.*?)\n\}", text, re.S):
        found = set()
        for _, _, body in attributes(m.group(2)):
            items = split_top(body)
            keys = {key_of(i): i for i in items}
            if "arg" in keys and "name" in keys:
                found.update(strings_in(keys["name"]))
            for item in items:
                k = key_of(item)
                if k in ("long", "alias"):
                    found.update("--" + s for s in strings_in(item))
                elif k == "short":
                    found.add("-" + re.search(r"'(.)'", item).group(1))
        names[(m.start(), m.end())] = found
    return names


def main():
    text = SOURCE.read_text()
    structs = owners(text)
    dropped = 0
    out, last = [], 0
    for start, end, body in attributes(text):
        span = next(((a, b) for (a, b) in structs if a <= start < b), None)
        local = structs.get(span, set())
        items = split_top(body)
        is_arg = any(i == "arg" for i in items)
        mapped = []
        for item in items:
            k = key_of(item)
            if k in DROP:
                continue
            if k == "arg":
                mapped.append("positional")
            elif k == "name":
                mapped.append(item.replace("name", "value_name", 1) if is_arg else item)
            elif k == "arg_required_else_help":
                mapped.append("arg_required_else_help")
            elif k in SELECTORS:
                keep = [s for s in strings_in(item) if s in local]
                dropped += len(strings_in(item)) - len(keep)
                if keep:
                    mapped.append(k + "(" + ", ".join(f'"{s}"' for s in keep) + ")")
            elif k in KEEP or k == "group":
                mapped.append(item)
            else:
                sys.exit(f"unknown usage attribute `{k}` in: {item}")
        out.append(text[last:start])
        if mapped:
            out.append("#[arg(" + ", ".join(mapped) + ")]")
        else:
            # Nothing left: drop the attribute and the newline after it.
            if text[end : end + 1] == "\n":
                end += 1
            while out[-1].endswith(" "):
                out[-1] = out[-1][:-1]
        last = end
    out.append(text[last:])
    result = "".join(out)
    result = result.replace(
        "use usage_derive::{Args, Cli, Subcommands, ValueEnum};",
        "use winnow_args::{Args, Subcommand, ValueEnum};",
    )
    # The doc comments are mise's help text, not rustdoc: `[internal]` is not a link.
    result = result.replace(
        "#![allow(dead_code, unused_imports)]",
        "#![allow(dead_code, unused_imports)]\n"
        "// mise's examples are indented like code blocks; they are not doctests.\n"
        "#![cfg(not(doctest))]\n"
        "#![allow(\n    rustdoc::broken_intra_doc_links,\n    rustdoc::bare_urls,\n    rustdoc::invalid_html_tags,\n    rustdoc::invalid_rust_codeblocks\n)]",
    )
    result = result.replace("#[derive(Cli)]", "#[derive(Args)]")
    result = result.replace("#[derive(Subcommands)]", "#[derive(Subcommand)]")
    # usage's default: a flag-like word naming no flag is a positional value.
    result = result.replace("#[derive(Args)]\n", "#[derive(Args)]\n#[arg(unknown_flags = \"value\")]\n")
    result = result.replace(
        "A shadow of `mise.usage.kdl` in usage's vocabulary, generated by\n//! `cargo run -p xtask -- gen-shadow`.",
        "A shadow of `mise.usage.kdl` in winnow-args' vocabulary, translated from\n"
        "//! usage's shadow by `tasks/gen-mise-shadow.py`.",
    )
    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text(result)
    subprocess.run(["rustfmt", "--edition", "2024", str(TARGET)], check=True)
    print(f"wrote {TARGET.relative_to(ROOT)}; dropped {dropped} selector(s) naming another struct's flag")


if __name__ == "__main__":
    main()
