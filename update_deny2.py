import sys

with open('deny.toml', 'r') as f:
    content = f.read()

replacement = """  "RUSTSEC-2026-0066",
  # RUSTSEC-2026-0049 (rustls-webpki)
  # - Path: bollard -> rustls-webpki
  # - Risk: testcontainers/bollard lock to older rustls-webpki versions
  # - Action: Ignore until dependencies update
  "RUSTSEC-2026-0049"
]"""

if '  "RUSTSEC-2026-0066"\n]' in content:
    content = content.replace('  "RUSTSEC-2026-0066"\n]', replacement)
    with open('deny.toml', 'w') as f:
        f.write(content)
    print("Updated deny.toml again")

with open('.cargo/audit.toml', 'r') as f:
    audit_content = f.read()

audit_replacement = """  "RUSTSEC-2026-0066",
  # RUSTSEC-2026-0049: rustls-webpki issue - dev-only via bollard
  "RUSTSEC-2026-0049"
]"""

if '  "RUSTSEC-2026-0066"\n]' in audit_content:
    audit_content = audit_content.replace('  "RUSTSEC-2026-0066"\n]', audit_replacement)
    with open('.cargo/audit.toml', 'w') as f:
        f.write(audit_content)
    print("Updated audit.toml again")
