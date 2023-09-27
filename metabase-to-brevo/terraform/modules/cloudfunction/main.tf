resource "google_cloudfunctions2_function" "main" {
  name        = var.name
  location    = var.location
  description = var.description
  project     = var.project

  build_config {
    runtime     = var.runtime
    entry_point = var.entrypoint

    source {
      storage_source {
        bucket = google_storage_bucket.main.name
        object = google_storage_bucket_object.main.name
      }
    }
  }

  service_config {
    min_instance_count    = var.min_instance_count
    max_instance_count    = var.max_instance_count
    available_memory      = "256M"
    timeout_seconds       = 540
    service_account_email = google_service_account.main.email

    environment_variables = var.environment_variables

    dynamic "secret_environment_variables" {
      for_each = var.secret_environment_variables

      content {
        key        = secret_environment_variables.key
        project_id = var.project
        secret     = google_secret_manager_secret.main[secret_environment_variables.key].secret_id
        version    = "latest"
      }
    }
  }

  depends_on = [google_secret_manager_secret_iam_member.main]
}
