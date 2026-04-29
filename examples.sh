#!/bin/bash

# Address Wise API Examples

# 1. Resolve Address - Simple Query
# This uses the 'query' field for a free-text search.
curl -X POST http://localhost:8080/resolve-address \
  -H "Content-Type: application/json" \
  -d '{
    "query": "1600 Pennsylvania Avenue NW, Washington, DC 20500"
  }'

# 2. Resolve Address - Structured Input
# This uses individual fields for a more precise search.
curl -X POST http://localhost:8080/resolve-address \
  -H "Content-Type: application/json" \
  -d '{
    "street": "Pennsylvania Avenue NW",
    "houseNumber": "1600",
    "city": "Washington",
    "postalCode": "20500",
    "countryCode": "US"
  }'

# 3. Resolve Address - Nested Structured Input
# The API also supports a nested 'address' object.
curl -X POST http://localhost:8080/resolve-address \
  -H "Content-Type: application/json" \
  -d '{
    "address": {
      "street": "Broadway",
      "houseNumber": "1",
      "city": "New York",
      "countryCode": "US"
    }
  }'

# 4. Autocomplete - Session-aware street prefix search
curl -X POST http://localhost:8080/autocomplete \
  -H "Content-Type: application/json" \
  -d '{
    "sessionId": "demo-session-1",
    "query": "aven",
    "countryBias": "FR",
    "limit": 10
  }'

# 5. Autocomplete Sandbox
# Open http://localhost:8080/sandbox/autocomplete in a browser.
