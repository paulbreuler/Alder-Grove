# Health endpoint returns ok
GET http://localhost:3001/health
HTTP 200
Content-Type: application/json
[Asserts]
jsonpath "$.status" == "ok"
