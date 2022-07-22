Hook0 API
=========

# Setup dev-env

- Spawn a local postgresql server checkout [database](../database)
- Setup database url in `.env`
- Start API

```bash
cargo run --bin api
```

## Updating queries

sqlx-cli is required to update prepared statements

```bash
cargo install sqlx-cli
cargo sqlx prepare
```

### LICENSE

Hook0 is free and the source is available. Versions are published under
the [Server Side Public License (SSPL) v1](./LICENSE.txt).

The license allows the free right to use, modify, create derivative works, and redistribute, with three simple
limitations:

- You may not provide the products to others as a managed service
- You may not circumvent the license key functionality or remove/obscure features protected by license keys
- You may not remove or obscure any licensing, copyright, or other notices
