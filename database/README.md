# Start database for local development

Since we edit directly `schema.sql`, use the following command to always recreate the db from scratch:

```
rm -rf ./postgres && mkdir -p ./postgres && docker compose up --force-recreate
```
