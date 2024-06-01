## .env file
```text
HOST=0.0.0.0
DATABASE_URL=postgres://portfolio:example@postgres/portfolio
OPENAI_API_KEY=foo
```

## Prompt Errors
- Answer
  - auto setting answers after getting questions
  - not setting answers after getting responses


## TODO
- [x] generate result
- [x] add tokens limit
- [x] real database integration
- [x] rest api
  - [x] endpoints
  - [x] customization dialogue params
  - [x] build docker
  - [ ] move to separated dir
- [ ] pdf generation
- [ ] if first message will be too long (it's skip limit now)
- [ ] interface
  - [ ] discord
  - [ ] telegram
  - [ ] web
- [ ] real expectations
- [ ] add abstraction level for use different AI API/local (Google/OpenAI/Llama)