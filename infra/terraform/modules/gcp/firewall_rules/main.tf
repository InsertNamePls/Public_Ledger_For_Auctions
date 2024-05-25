resource "google_compute_firewall" "ingress_rules" {
  name    = "${var.rule_name}-ingress"
  network = var.vpc_id
  direction = "INGRESS"
  project = var.project_id
  source_ranges = var.source_ranges
  allow {
    protocol =  var.protocol
    ports    =  var.ports
  }
  target_tags   = ["ssh"]
}

resource "google_compute_firewall" "egress_rules" {
  name    = "${var.rule_name}-egress"
  network = var.vpc_id
  direction = "EGRESS"
  project = var.project_id
  source_ranges = var.desitnation_ranges
  allow {
    protocol =  var.protocol
    ports    =  var.ports
  }
}

