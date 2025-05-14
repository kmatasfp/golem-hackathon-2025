### Lambdaconf 2025 golem hackathon project

Can ask following queries

Using repl
```
let worker = instance("worker")
worker.prompt("what are the golem cloud supported languages based on https://learn.golem.cloud/develop-overview")
```

Or http client like `xh`
```
xh post localhost:9006/conversations/conv1/prompts input="what are the golem cloud supported languages based on https://learn.golem.cloud/develop-overview"
```
