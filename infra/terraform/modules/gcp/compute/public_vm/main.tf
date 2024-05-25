resource "google_compute_instance" "vm" {
  count        = var.num_instances
  name         = "${var.vm_name}${count.index}"
  machine_type = var.machine_type
  zone         = var.available_zones[count.index]

  boot_disk {
    initialize_params {
      image = var.image
    }
  }

  metadata_startup_script = "apt update -y && apt-get install ${var.packages} -y"

  scheduling {
    provisioning_model = var.provisioning_model
    preemptible        = true
    automatic_restart  = false
  }

  network_interface {
    network = var.vpc_id
    access_config {}
    subnetwork = var.subnet
  }

  tags = var.tags

  metadata = {
    "ssh-keys" = "${var.username}:${var.ssh_pub} ${var.username}:"
  }
  service_account {
    # Google recommends custom service accounts that have cloud-platform scope and permissions granted via IAM Roles.
    email  = var.defaul_sa_name
    scopes = var.scopes
  }
}
