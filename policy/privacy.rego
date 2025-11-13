package privacy
deny[msg]{f:=input.fields[_]; f.pii!=""; not f.owner; msg:=sprintf("PII %s missing owner",[f.path])}
