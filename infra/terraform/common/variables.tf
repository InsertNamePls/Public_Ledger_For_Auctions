variable "tfstate_bucket_name" {}
variable "project_name" {}
variable "project_id" {}
variable "region" {}
variable "ip_isp_pub" {}
variable "path_local_public_key" {
  sensitive = true
}
variable "username" {}
variable "scopes" {}
variable "image" {}
variable "bootstrap_node_machine_type" {}
variable "bootstrap_node_provisioning_model" {}
variable "bootstrap_node_tags" {}
variable "auction_client_n_nodes" {}
variable "auction_client_machine_type" {}
variable "auction_client_provisioning_model" {}
variable "auction_client_tags" {}
variable "auctions_server_n_nodes" {}
variable "auctions_server_machine_type" {}
variable "auctions_server_provisioning_model" {}
variable "auctions_server_tags" {}
variable "service_account_file" {}
