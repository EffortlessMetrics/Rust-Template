package flags
deny[msg]{f:=input.flags[_]; not f.owner; msg:=sprintf("Flag %s has no owner",[f.key])}
