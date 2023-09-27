data "archive_file" "main" {
  type        = "zip"
  source_dir  = var.source_dir
  output_path = "${var.output_path}/${var.name}.zip"
  excludes    = var.excludes
}

resource "google_storage_bucket" "main" {
  name                        = "${var.project}-${var.name}-source"
  location                    = var.location
  uniform_bucket_level_access = true
}

resource "google_storage_bucket_object" "main" {
  name   = "${var.name}-${data.archive_file.main.output_md5}.zip"
  bucket = google_storage_bucket.main.name
  source = data.archive_file.main.output_path
}
