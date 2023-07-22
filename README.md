# qe

a simple queueing system for ai requests

```bash
curl -X POST \
  http://localhost:8000/ai/push \
  -H 'cache-control: no-cache' \
  -H 'content-type: application/json' \
  -d '{
        "text": "what color is the sky?"
}'
```
