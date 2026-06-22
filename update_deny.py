import sys

with open('deny.toml', 'r') as f:
    content = f.read()

replacement = """  "RUSTSEC-2025-0134",
  # RUSTSEC-2026-0066 (astral-tokio-tar)
  # - Path: testcontainers -> astral-tokio-tar
  # - Risk: testcontainers locks to astral-tokio-tar 0.5.x, blocking updates
  # - Action: Ignore until testcontainers updates
  "RUSTSEC-2026-0066"
]"""

if '  "RUSTSEC-2025-0134"\n]' in content:
    content = content.replace('  "RUSTSEC-2025-0134"\n]', replacement)
    with open('deny.toml', 'w') as f:
        f.write(content)
    print("Updated deny.toml")
else:
    print("Could not find target block in deny.toml")

with open('.cargo/audit.toml', 'r') as f:
    audit_content = f.read()

audit_replacement = """  "RUSTSEC-2025-0134",
  # RUSTSEC-2026-0066: astral-tokio-tar unmaintained - dev-only via testcontainers
  "RUSTSEC-2026-0066"
]"""

if '  "RUSTSEC-2025-0134"\n]' in audit_content:
    audit_content = audit_content.replace('  "RUSTSEC-2025-0134"\n]', audit_replacement)
    with open('.cargo/audit.toml', 'w') as f:
        f.write(audit_content)
    print("Updated audit.toml")
else:
    print("Could not find target block in audit.toml")
