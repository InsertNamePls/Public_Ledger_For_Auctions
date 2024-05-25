resource "google_compute_disk" "storage_device" {
  count = var.storage_device_number
  name = "${var.storage_device_name}-${count.index}"
  type = var.storage_device_type
  zone = element(var.available_zones, count.index)
  size = var.storage_device_size
}