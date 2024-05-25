terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "5.0.0"
    }
  }
}


provider "google" {
  credentials = file(var.service_account_file)
  project     = var.project_id
  region      = var.region
}
