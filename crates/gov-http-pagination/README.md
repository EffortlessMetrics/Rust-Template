# gov-http-pagination

Single-responsibility pagination primitives used by governance HTTP crates.

## Scope

- Pagination query parameters (`PaginationParams`)
- Pagination metadata (`Pagination`)
- Generic paginated response wrapper (`PaginatedResponse<T>`)

## Why this crate exists

Pagination is shared by multiple governance subrouters and repositories. Splitting it into a
microcrate keeps the contract reusable and independent from routing, file I/O, and error handling.
