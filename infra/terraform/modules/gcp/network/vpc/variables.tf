variable "project_name" {
    type = string
}
variable "vpc_name" {
    type = string
}
variable "auto_create_subnetworks" {
    type=bool
}
variable "delete_default_routes_on_create" {
    type = bool
}
variable "routing_mode" {
    type = string
}
variable "route_name" {
  type = string
}
variable "route_priority" {
  type = number
}
variable "dest_ip_range" {
  type = string
}
variable "next_hop_gateway"{
    type = string
}