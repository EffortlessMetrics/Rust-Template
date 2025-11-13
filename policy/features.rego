package features
deny[msg]{f:=input.features[_]; ac:=f.acceptance_criteria[_]; not ac_in_ledger(ac,input.ledger_ac_ids); msg:=sprintf("Feature %s references unknown AC %s",[f.id,ac])}
ac_in_ledger(x,ids){x==ids[_]}
