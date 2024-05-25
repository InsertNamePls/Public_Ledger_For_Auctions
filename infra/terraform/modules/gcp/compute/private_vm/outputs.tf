output "vm_ids"{
    value = google_compute_instance.vm.*.id
}