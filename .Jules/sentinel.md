## 2024-05-15 - Missing Vary header with Access-Control-Allow-Origin
**Vulnerability:** When dynamically returning an Access-Control-Allow-Origin header based on the incoming Origin header, without a Vary: Origin header, intermediate caches might incorrectly cache the CORS response and serve it to other origins.
**Learning:** Returning Access-Control-Allow-Origin dynamically without Vary: Origin exposes the application to cache poisoning.
**Prevention:** Always append Vary: Origin when Access-Control-Allow-Origin dynamically reflects the incoming Origin header.
