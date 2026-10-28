#!/usr/bin/env python3
"""Render README.md from catalog.json. Run from anywhere: python3 render.py"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))


def esc(text):
    return (text or "").replace("|", "\\|")


def first_date(browser):
    return browser["versions"][0]["date"] if browser["versions"] else ""


def render(cat):
    out = []
    w = out.append
    w("# 1990s web browsers: a screenshot catalog")
    w("")
    w("<!-- Generated from catalog.json by render.py. Edit those, not this file. -->")
    w("")
    w(cat["about"])
    w("")
    w("## Rules for using this material")
    w("")
    for rule in cat["rules"]:
        w(f"- {rule}")
    w("")
    w("## At a glance")
    w("")
    w("| Browser | First release | Platforms | In this repo | Linked |")
    w("| --- | --- | --- | --- | --- |")
    for b in cat["browsers"]:
        w(f"| [{esc(b['name'])}](#{b['id']}) | {first_date(b)} | {esc(', '.join(b['platforms']))} "
          f"| {len(b['screenshots'])} | {len(b['linked'])} |")
    w("")
    for b in cat["browsers"]:
        w(f'<a id="{b["id"]}"></a>')
        w("")
        w(f"## {b['name']}")
        w("")
        w(f"{b['maker']} · {', '.join(b['platforms'])} · [Wikipedia]({b['source']})")
        w("")
        w(f"**Look:** {b['look']}")
        w("")
        w("| Version | Released | Notes |")
        w("| --- | --- | --- |")
        for v in b["versions"]:
            w(f"| {esc(v['version'])} | {v['date']} | {esc(v['notes'])} |")
        w("")
        for s in b["screenshots"]:
            w(f'<img src="{s["file"]}" width="560" alt="{b["name"]} {s["version"]}">')
            w("")
            w(f"*{s['version']}:* {s['shows']}  ")
            w(f"Credit: {s['author']} · [{s['license']}]({s['license_url'] or s['source_url']}) · "
              f"[source]({s['source_url']})")
            w("")
        if b.get("to_copy"):
            w("Freely licensed on Commons, not copied here yet:")
            w("")
            for t in b["to_copy"]:
                w(f"- [{t['commons_file']}]({t['source_url']}) ({t['license']}, {t['author']}): {t['shows']}")
            w("")
        if b["linked"]:
            w("Fair-use screenshots on Wikipedia (linked, not copied):")
            w("")
            for l in b["linked"]:
                w(f"- [{l['title']}]({l['source_url']}): {l['shows']}")
            w("")
        for label, url in b.get("try", []):
            w(f"Try it: [{label}]({url})")
            w("")
    w("## No screenshot found yet")
    w("")
    w("Browsers of the decade that still need a screenshot. A freely licensed one could be added here.")
    w("")
    w("| Browser | First release |")
    w("| --- | --- |")
    for g in cat["gaps"]:
        w(f"| [{esc(g['name'])}]({g['source']}) | {g['date']} |")
    w("")
    w("## References")
    w("")
    for r in cat["references"]:
        w(f"- [{r['label']}]({r['url']})")
    w("")
    w("## Updating this catalog")
    w("")
    w("1. Find a screenshot on [Wikimedia Commons](https://commons.wikimedia.org/) and check its file page:")
    w("   copy it into `images/` only if it is public domain or under a free licence")
    w("   (CC0, CC BY, CC BY-SA, GPL, FAL and similar). Anything marked fair use or")
    w("   non-free goes in `linked` instead.")
    w("2. Add it to `catalog.json` with its author, licence, licence URL and source page.")
    w("   A free file found but not downloaded yet goes in `to_copy`; move it to")
    w("   `screenshots` once it is in `images/` and you have checked it shows the browser itself.")
    w("3. Run `python3 render.py` to regenerate this file.")
    w("")
    return "\n".join(out)


def main():
    with open(os.path.join(HERE, "catalog.json"), encoding="utf-8") as f:
        cat = json.load(f)
    with open(os.path.join(HERE, "README.md"), "w", encoding="utf-8") as f:
        f.write(render(cat))


if __name__ == "__main__":
    main()
