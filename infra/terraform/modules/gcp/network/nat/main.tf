resource "google_compute_router" "router" {
  provider = google-beta
  project  = var.project_name
  name     = var.router_name
  region   = var.region
  network  = var.vpc_id
}

resource "google_compute_router_nat" "nat" {
  name                               = var.router_name
  router                             = google_compute_router.router.name
  region                             = var.region
  nat_ip_allocate_option             = var.allocate_option
  source_subnetwork_ip_ranges_to_nat = var.ranges_to_nat
  
  depends_on = [google_compute_router.router]
}