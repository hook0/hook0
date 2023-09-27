resource "google_secret_manager_secret" "main" {
  for_each = var.secret_environment_variables

  secret_id = each.key

  replication {
    user_managed {
      replicas {
        location = var.location
      }
    }
  }
}

resource "google_secret_manager_secret_version" "main" {
  for_each = var.secret_environment_variables

  secret = google_secret_manager_secret.main[each.key].name
  secret_data = each.value
  enabled     = true
}

resource "google_secret_manager_secret_iam_member" "main" {
  for_each = var.secret_environment_variables

  project   = google_secret_manager_secret.main[each.key].project
  secret_id = google_secret_manager_secret.main[each.key].secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.main.email}"
}
