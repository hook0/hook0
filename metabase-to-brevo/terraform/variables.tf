variable "project" {
  description = "(Required) The ID of the project in which the resource belongs."
  type        = string
}

variable "backend" {
  description = "(Required) The backend where Terraform stores its state data files."
  type        = string
}

variable "region" {
  type        = string
  description = "(Optional) Project region. If not provided, defaults to europe-west1"
  default     = "europe-west1"
}

variable "availability_zone" {
  type        = string
  description = "(Optional) Project availability zone. If not provided, defaults to europe-west1-b"
  default     = "europe-west1-b"
}

variable "environment_variables" {
  description = "(Required) Environment variables that shall be available during function execution"
  type        = map(string)
}

variable "secret_environment_variables" {
  description = "(Required) Secret environment variables configuration"
  type        = map(string)
}
