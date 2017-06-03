# esmozy

SpiderMonkey AST is largely compatible with [ESTree](https://github.com/estree/estree) in Rust, and can be made fully compatible by overriding AST builders via options.

SpiderMonkey, unlike all the other JavaScript engines, exposes its parser via a public API - [`Reflect.parse`](https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Parser_API#Reflect.parse(src_options)) - available even from their JavaScript console.

This repo is a prototype to get ESTree AST in Rust by literally embedding SpiderMonkey and invoking JavaScript that calls `Reflect.parse` and passes result back in JSON format, which is easily consumable by Rust side or can be passed further.

As turned out, *(surprise!)* speed is not appealing, but it might work for prototyping JavaScript tooling in Rust until better parsers are available.
