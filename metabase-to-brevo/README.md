# Metabase to SendInBlue connector

Synchronize Metabase views with SendInBlue contact lists

# Development

## Environment variables

Copy `.env.dist` to `.env` and fill with your own values.

## Install dependencies

```shell
npm install
```

## Start

```shell
npm run build && npm start:run
```

---

# Production

## Environment variables

Copy `terraform/tfvars/sample.tfvars` to `<workspace>.tfvars` and fill with your own values.

## Provisioning

```shell
cd terraform
tf init
tf workspace select <workspace>
tf apply -var-file=tfvars/<workspace>.tfvars
```
