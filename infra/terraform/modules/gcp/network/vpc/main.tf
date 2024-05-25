resource "google_compute_network" "vpc" {
  name                              = var.vpc_name
  auto_create_subnetworks           = var.auto_create_subnetworks
  project                           = var.project_name
  delete_default_routes_on_create   = var.delete_default_routes_on_create
  routing_mode                      = var.routing_mode
}

resource "google_compute_route" "route" {
  name              = var.route_name
  dest_range        = var.dest_ip_range
  network           = google_compute_network.vpc.id
  priority          = var.route_priority
  next_hop_gateway  = var.next_hop_gateway
}

