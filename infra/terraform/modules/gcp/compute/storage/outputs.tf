output "volume_names" {
    value = google_compute_disk.storage_device.*.name
}
output "volume_ids" {
    value = google_compute_disk.storage_device.*.id
}
