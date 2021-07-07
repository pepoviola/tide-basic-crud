# tide-basic-crud
Basic CRUD api using [rust](https://www.rust-lang.org/) and [tide](https://github.com/http-rs/tide) with  postgresql.

The code of this repo is also commented/explained in this [post posts](https://javierviola.com/tags/tide/), where you can read from the initial implementation (using in-memory HashMap as db) and follow the progress and refactors made in the code.

### Stack

- Tide
- sqlx
- Tera

### CI/CD
 - GH Actions for CI
 - I currently using [dokku](https://github.com/dokku/dokku) and you can find the working environment at https://tide-basic-crud.labs.javierviola.com/


TODO:
[] Schema validation
[] Move front-end to wasm (localghost/yew)
[] Add telemetry
[] Better error handling
[] Add other resources (entities) and refactor structure.
[] Add tutorial on how to run locally
