#!/usr/bin/env python3
import sys
from pathlib import Path

# Try importing yaml, if not present, we might need to install it or warn
try:
    import yaml  # pip install pyyaml
except ImportError:
    print("Error: pyyaml is required. Install it with 'pip install pyyaml'")
    sys.exit(1)

ROOT = Path(__file__).resolve().parents[1]
SKILLS_DIR = ROOT / ".claude" / "skills"


def iter_skill_files():
    if not SKILLS_DIR.exists():
        return
    for skill_dir in SKILLS_DIR.iterdir():
        if not skill_dir.is_dir():
            continue
        skill_file = skill_dir / "SKILL.md"
        if skill_file.exists():
            yield skill_dir.name, skill_file


def split_frontmatter(content: str):
    lines = content.splitlines(keepends=True)
    if not lines or not lines[0].strip() == "---":
        return None, content  # no frontmatter yet

    # find closing ---
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            fm = "".join(lines[1:i])
            body = "".join(lines[i + 1 :])
            return fm, body
    # unterminated frontmatter
    return None, content


def format_skill(slug: str, path: Path) -> bool:
    original = path.read_text(encoding="utf-8")
    fm_str, body = split_frontmatter(original)

    if fm_str is None:
        # synthesize minimal frontmatter if missing
        fm = {
            "name": slug,
            "description": f"Skill for {slug}. Please update description.",
        }
    else:
        try:
            fm = yaml.safe_load(fm_str) or {}
        except yaml.YAMLError:
            # If YAML is invalid, we can't format it safely without potentially destroying it.
            # We'll leave it for the linter to catch.
            return False

    # ensure required fields
    fm.setdefault("name", slug)
    fm.setdefault(
        "description",
        f"Skill for {slug}. Please update description.",
    )

    # normalize keys order
    ordered_keys = ["name", "description", "allowed-tools"]
    ordered = {k: fm.get(k) for k in ordered_keys if k in fm}
    # keep any extra keys in stable order at the end
    for k in fm:
        if k not in ordered:
            ordered[k] = fm[k]

    new_fm_str = yaml.safe_dump(
        ordered,
        sort_keys=False,
        default_flow_style=False,
    )

    # normalize body: ensure exactly one blank line after frontmatter
    body = body.lstrip("\n")  # strip leading blank lines
    new_content = "---\n" + new_fm_str + "---\n\n" + body

    if new_content != original:
        path.write_text(new_content, encoding="utf-8")
        return True
    return False


def main():
    changed = False
    for slug, path in iter_skill_files():
        if format_skill(slug, path):
            print(f"formatted {path}")
            changed = True
    return 1 if changed else 0  # non-zero so pre-commit re-runs hooks


if __name__ == "__main__":
    sys.exit(main())
