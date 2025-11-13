#!/usr/bin/env bash
set -euo pipefail
git config user.name "gha-bot"
git config user.email "gha-bot@users.noreply.github.com"
branch="maintenance/pin-actions"
git checkout -B "$branch"
changed=0
for f in .github/workflows/*.yml .github/workflows/*.yaml; do
  [[ -e "$f" ]] || continue
  while read -r owner repo ref; do
    sha=$(gh api "repos/$owner/$repo/commits/$ref" --jq '.sha' 2>/dev/null       || gh api "repos/$owner/$repo/git/ref/tags/$ref" --jq '.object.sha' 2>/dev/null || true)
    [[ -n "$sha" ]] || continue
    sed -i.bak -E "s#(uses:\s*$owner/$repo)@[^[:space:]]+#\1@$sha#g" "$f" && rm -f "$f.bak"
    changed=1
  done < <(grep -Eho 'uses:\s*([[:alnum:]._-]+)/([[:alnum:]._-]+)@([[:alnum:]._/-]+)' "$f"            | sed -E 's/uses:\s*([^/]+)\/([^@]+)@(.+)/\1 \2 \3/'            | grep -vE '@[0-9a-f]{40}$')
done
if [[ "$changed" == "1" ]]; then
  git add .github/workflows
  git commit -m "Pin GitHub Actions to commit SHAs"
  git push -u origin "$branch" --force
  gh pr create --fill --title "Pin Actions to SHAs" --body "Automated hard-pinning of GitHub Actions."
else
  echo "No changes."
fi
