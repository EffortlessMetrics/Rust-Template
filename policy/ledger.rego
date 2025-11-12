package ledger
deny[msg] {
  some s, r, ac
  s := input.stories[_]
  r := s.requirements[_]
  ac := r.acceptance_criteria[_]
  count(ac.tests) == 0
  msg := sprintf("AC %s has no mapped tests", [ac.id])
}
