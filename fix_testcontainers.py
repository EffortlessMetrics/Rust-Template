import sys

with open("crates/adapters-db-sqlx/Cargo.toml", "r") as f:
    content = f.read()

content = content.replace('testcontainers = "0.26.0"', 'testcontainers = "0.27.3"')

with open("crates/adapters-db-sqlx/Cargo.toml", "w") as f:
    f.write(content)
print("Updated testcontainers to 0.27.3")
