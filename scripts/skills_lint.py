#!/usr/bin/env python3
import re
import sys
from pathlib import Path

# Try importing yaml
try:
    import yaml
except ImportError:
    print("Error: pyyaml is required. Install it with 'pip install pyyaml'")
    sys.exit(1)

ROOT = Path(__file__).resolve().parents[1]
SKILLS_DIR = ROOT / ".claude" / "skills"
NAME_RE = re.compile(r"^[a-z0-9-]{1,64}$")


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
    if not lines or lines[0].strip() != "---":
        raise ValueError("Missing frontmatter '---' at line 1")
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            fm = "".join(lines[1:i])
            body = "".join(lines[i + 1 :])
            return fm, body
    raise ValueError("Unterminated frontmatter; missing closing '---'")


def lint_skill(slug: str, path: Path) -> list[str]:
    errors: list[str] = []

    content = path.read_text(encoding="utf-8")
    if "\t" in content.split("\n", 20)[0]:  # quick check first lines for tabs
        errors.append("Tabs found in SKILL.md (YAML must use spaces).")

    try:
        fm_str, body = split_frontmatter(content)
    except ValueError as e:
        errors.append(str(e))
        return errors

    try:
        fm = yaml.safe_load(fm_str) or {}
    except yaml.YAMLError as e:
        errors.append(f"YAML parse error: {e}")
        return errors

    name = fm.get("name")
    desc = fm.get("description")

    if not isinstance(name, str):
        errors.append("frontmatter 'name' must be a string.")
    else:
        if not NAME_RE.match(name):
            errors.append(
                f"frontmatter 'name' must match {NAME_RE.pattern} (got '{name}')."
            )
        if name != slug:
            errors.append(
                f"frontmatter 'name' ('{name}') must equal directory slug ('{slug}')."
            )

    if not isinstance(desc, str) or not desc.strip():
        errors.append("frontmatter 'description' must be a non-empty string.")
    elif len(desc) > 1024:
        errors.append("frontmatter 'description' must be ≤1024 characters.")

    allowed = fm.get("allowed-tools")
    if allowed is not None and not isinstance(allowed, list):
        errors.append("frontmatter 'allowed-tools' must be a YAML list if present.")

    # Body must have at least one heading
    if "#" not in body:
        errors.append("Markdown body should contain at least one heading (# …).")

    return errors


def main():
    any_errors = False
    for slug, path in iter_skill_files():
        errors = lint_skill(slug, path)
        if errors:
            any_errors = True
            rel = path.relative_to(ROOT)
            print(f"[SKILL LINT] {rel}:")
            for e in errors:
                print(f"  - {e}")
            print()
    return 1 if any_errors else 0


if __name__ == "__main__":
    sys.exit(main())
