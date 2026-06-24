#!/bin/bash
cat crates/http-middleware/src/security_headers.rs | grep -n "HeaderValue::from_str"
