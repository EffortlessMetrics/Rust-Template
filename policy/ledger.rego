package ledger
deny[msg]{some s,sr,ac; s:=input.stories[_]; sr:=s.requirements[_]; ac:=sr.acceptance_criteria[_]; count(ac.tests)==0; msg:=sprintf("AC %s has no mapped tests",[ac.id])}
