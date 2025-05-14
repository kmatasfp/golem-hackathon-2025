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

For making it agent like and making it answer questions regarding content on `learn.golem.cloud` without providing the doc ref link within the prompt

One needs to do following manually atm like so:
```
worker.add-context("https://learn.golem.cloud/fundamentals")
worker.add-context("https://learn.golem.cloud/quickstart")
worker.add-context("https://learn.golem.cloud/concepts/reliability")
worker.add-context("https://learn.golem.cloud/concepts/components")
worker.add-context("https://learn.golem.cloud/concepts/workers")
...
```
