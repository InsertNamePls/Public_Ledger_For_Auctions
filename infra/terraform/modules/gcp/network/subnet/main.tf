resource "google_compute_subnetwork" "subnet" {
  name             = var.subnet_name
  region           = var.region
  ip_cidr_range    = var.ip_cidr
  purpose          = var.subnet_purpose
  network          = var.vpc_id
}
