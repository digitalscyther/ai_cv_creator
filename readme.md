## .env file
api:
```text
HOST=0.0.0.0
DATABASE_URL=postgres://portfolio:example@postgres/portfolio
OPENAI_API_KEY=foo
MDPROOF_FP=/bin/mdproof
MINIO_URL=http://minio:9000
MINIO_ACCESS_KEY=<access_key>
MINIO_SECRET_KEY=<secret_key>
MINIO_BUCKET_NAME=<bucket_name>
```

telegram:
```text
DATABASE_URL=postgres://portfolio:example@postgres/telegram
BOT_TOKEN=<bot_token>
BOT_NAME=<bot_name>
API_URL=http://api:3000
```

## Prompt Errors
- Answer
  - ~~auto setting answers after getting questions~~
  - ~~not setting answers after getting responses~~


## TODO
- [x] generate result
- [x] add tokens limit
- [x] real database integration
- [x] rest api
  - [x] endpoints
  - [x] customization dialogue params
  - [x] build docker
  - [x] move to separated dir
- [ ] pdf generation
  - [x] s3 work
  - [ ] send to telegram
  - [ ] normal format (prompt)
- [ ] if first message will be too long (it's skip limit now)
- [x] interface
  - [x] telegram
  - [ ] ~~discord~~
  - [ ] ~~web~~
- [ ] real expectations
- [ ] add abstraction level for use different AI API/local (Google/OpenAI/Llama)
