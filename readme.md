## .env file
api:
```text
HOST=0.0.0.0
DATABASE_URL=postgres://portfolio:example@postgres/portfolio
OPENAI_API_KEY=foo
PROGRAM_FP=/bin/wkhtmltopdf
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
  - Sometimes write N/A response as null. And this stop process.
  - Not autostart generate PDF
  - Small tokens limit spent (need about 200k minimum)


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
  - [x] reset with delete saved
  - [x] add to telegram
  - [x] handle "generated" answer via tg
  - [ ] ~~save original markdown~~
  - [ ] normal format (prompt)
- [ ] if first message will be too long (it's skip limit now)
- [x] interface
  - [x] telegram
    - [ ] hide admin commands
    - [ ] telegram crash if bad api response (1)
  - [ ] ~~discord~~
  - [ ] ~~web~~
- [ ] ~~stream file instead cache~~
- [x] file saving into temporary file
- [ ] real expectations
- [ ] add abstraction level for use different AI API/local (Google/OpenAI/Llama)

(1)
```text
2024-06-06 01:30:13 thread 'tokio-runtime-worker' panicked at src/main.rs:124:67:
2024-06-06 01:30:13 called `Result::unwrap()` on an `Err` value: reqwest::Error { kind: Request, url: Url { scheme: "http", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("api")), port: Some(3000), path: "/users/5/message", query: None, fragment: None }, source: hyper_util::client::legacy::Error(SendRequest, hyper::Error(IncompleteMessage)) }
2024-06-06 01:30:13 note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
2024-06-06 01:32:38 thread 'main' panicked at /usr/local/cargo/registry/src/index.crates.io-6f17d22bba15001f/teloxide-0.12.2/src/dispatching/dispatcher.rs:410:43:
```