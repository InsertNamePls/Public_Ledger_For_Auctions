resource "google_compute_instance" "vm" {
    count = var.num_instances
    name         = "${var.vm_name}${count.index+1}"
    machine_type = var.machine_type
    zone = element(var.available_zones, count.index)
    boot_disk {
      initialize_params {
        image = var.image
      }
    }
    metadata_startup_script = "apt update -y && apt-get install ${var.packages} -y"

    scheduling {
        provisioning_model = var.provisioning_model
        preemptible = true
        automatic_restart = false
        instance_termination_action = "STOP"
    }
    network_interface {
      network = var.vpc_id
      subnetwork = var.subnet
      network_ip = var.static_ip
      access_config {}
    }
    tags = var.tags
    metadata = {
      "ssh-keys" =  "${var.username}:${var.ssh_pub} ${var.username}:"
    }
    service_account {
      # Google recommends custom service accounts that have cloud-platform scope and permissions granted via IAM Roles.
      email  = var.defaul_sa_name
      scopes = var.scopes
    }
    lifecycle {
      ignore_changes = [attached_disk]
    }
}