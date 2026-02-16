## 2024-05-22 - Pre-computing Constant Headers
**Learning:** `HeaderValue::from_str` involves parsing and validation overhead. When headers are constant (like security headers from config), re-parsing them on every request is wasted CPU cycles.
**Action:** Always pre-compute and cache `HeaderValue`s for static configuration in middleware, using `Arc` or similar to share them cheaply across requests.
