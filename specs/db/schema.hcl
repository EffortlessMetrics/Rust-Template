schema "public" {
  comment = "Task management schema"
}

table "tasks" {
  schema = schema.public
  comment = "Task entities with lifecycle status"

  column "id" {
    null = false
    type = uuid
    default = uuid_generate_v4()
    primary_key = true
  }

  column "title" {
    null = false
    type = varchar(255)
  }

  column "status" {
    null = false
    type = varchar(20)
    check = "status IN ('PENDING', 'COMPLETED')"
  }

  column "created_at" {
    null = false
    type = timestamp
    default = sql("now()")
  }
}

enum "task_status" {
  enum_value = "PENDING"
  enum_value = "COMPLETED"
}
