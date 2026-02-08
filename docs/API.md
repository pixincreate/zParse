# zParse API

Base URL: `http://127.0.0.1:3000`

## Endpoints

### Health
`GET /api/health`

Response:
```json
{"status":"ok"}
```

### Formats
`GET /api/formats`

Response:
```json
["json","toml","yaml","xml"]
```

### Parse
`POST /api/parse`

Request body:
```json
{"content":"...","format":"json"}
```

Response (success):
```json
{"status":"ok","data":{}}
```

Response (error):
```json
{"status":"error","error":"message"}
```

### Convert
`POST /api/convert`

Request body:
```json
{"content":"...","from":"json","to":"yaml"}
```

Response (success):
```json
{"status":"ok","content":"..."}
```

Response (error):
```json
{"status":"error","content":"message"}
```

## curl examples

Health:
```bash
curl -s http://127.0.0.1:3000/api/health
```

Formats:
```bash
curl -s http://127.0.0.1:3000/api/formats
```

Parse JSON:
```bash
curl -s -X POST http://127.0.0.1:3000/api/parse \
  -H "Content-Type: application/json" \
  -d '{"content":"{\"name\":\"test\"}","format":"json"}'
```

Convert JSON -> TOML:
```bash
curl -s -X POST http://127.0.0.1:3000/api/convert \
  -H "Content-Type: application/json" \
  -d '{"content":"{\"name\":\"test\"}","from":"json","to":"toml"}'
```
