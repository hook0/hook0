module "metabase_to_sendinblue_connector" {
  source      = "./modules/cloudfunction"
  name        = "metabase-to-sendinblue-connector"
  account_id  = "mtb-to-sib"
  project     = var.project
  location    = var.region
  description = "Synchronize Metabase views with SendInBlue contact lists"
  runtime     = "nodejs16"
  entrypoint  = "metabaseToSendInBlueConnector"
  source_dir  = "../dist"
  output_path = "../dist"
  excludes    = ["node_modules", ".env", ".env.dist"]

  environment_variables        = var.environment_variables
  secret_environment_variables = var.secret_environment_variables

  depends_on = [google_project_service.project_services]
}
