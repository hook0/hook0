output "name" {
  value = google_cloudfunctions2_function.main.name
}

output "uri" {
  value = google_cloudfunctions2_function.main.service_config[0].uri
}

output "service_account_email" {
  value = google_cloudfunctions2_function.main.service_config[0].service_account_email
}
