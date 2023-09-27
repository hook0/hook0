terraform {
  backend "gcs" {
    bucket = "fabienrenaud-tfstate"
  }
}