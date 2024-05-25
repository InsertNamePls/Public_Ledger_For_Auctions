
# data should be encrypted however that would increase the price  12$/month https://cloud.google.com/products/calculator#id=0a0af97b-f62e-4b5d-9588-13951b8d4af9
resource "google_storage_bucket" "bucket" {
  name          = var.name
  force_destroy = var.force_destroy
  #project = 
  location      = var.location
  storage_class = var.storage_class
  versioning {
    enabled = var.versioning
  }
}