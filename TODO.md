# TODO

Running list of things we want to do but haven't yet. Keep it committed.

## Infra / CI
- [ ] **Gate deploys on the frontend tests too.** CI (`.github/workflows/deploy.yml`) currently runs only `cargo test`; `npm run check` and `npm test` don't run, so a broken frontend test/type error can still deploy. Add a frontend step (`npm run check && npm test`) to the workflow before deploy.

## Reveal
- [ ] **Make card substitutions a configurable house rule in the reveal screen.** Right now the "represented by the Trickster card" / "represented by the Assassin card" proxies are hardcoded globally in `reveal.ts`. Let groups set their own role→physical-card substitutions (e.g. in the reveal screen / per namespace) instead of baking SGW's set into every game.
