## 2024-05-24 - Added aria-label to search inputs
**Learning:** Found that search inputs in the UI were missing accessibility attributes and type="search", making them less intuitive for screen readers and keyboard users. Also, filter buttons didn't communicate their active state correctly.
**Action:** Always add `aria-label` and use `type="search"` for search inputs. Ensure filter buttons update `aria-pressed` based on their active state.
