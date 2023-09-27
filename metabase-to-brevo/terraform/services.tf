resource "google_project_service" "project_services" {
  for_each = toset(["artifactregistry.googleapis.com", "secretmanager.googleapis.com"])

  project = var.project
  service = each.key

  disable_dependent_services = true
  disable_on_destroy         = false
}
