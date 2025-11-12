schema "public" {}

table "users" {
  schema = schema.public
  column "id"    { type = uuid; null = false }
  column "email" { type = text; null = false }
  primary_key { columns = [column.id] }
}
