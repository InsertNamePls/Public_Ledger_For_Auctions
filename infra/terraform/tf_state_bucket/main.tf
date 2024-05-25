module "tfstateBucket" {
  source        = "../modules/gcp/storage_bucket"
  name          = var.tfstate_bucket_name
  force_destroy = true
  location      = "EU"
  storage_class = "STANDARD"
  versioning    = true
}
