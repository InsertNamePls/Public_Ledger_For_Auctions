####################################################
###################### VPC #########################
####################################################
module "Network" {
  source = "../modules/gcp/network/vpc"

  project_name                    = var.project_id
  vpc_name                        = local.vpc_name
  auto_create_subnetworks         = false
  delete_default_routes_on_create = true
  routing_mode                    = "REGIONAL"
  route_name                      = "${local.vpc_name}-default-igw"
  next_hop_gateway                = "default-internet-gateway"
  route_priority                  = 1000
  dest_ip_range                   = "0.0.0.0/0"
}

####################################################
################## Private Subnet ##################
####################################################
module "PrivateAccessSubnet" {
  source = "../modules/gcp/network/subnet"

  vpc_id         = module.Network.vpc_id
  subnet_name    = local.private_subnet_name
  ip_cidr        = "10.10.0.0/24"
  subnet_purpose = "PRIVATE"
  region         = var.region

}
module "FirewallRulePrivate" {
  source = "../modules/gcp/firewall_rules"

  rule_name          = "private-network-rules"
  vpc_id             = module.Network.vpc_id
  protocol           = "tcp"
  ports              = ["22", "3000", "3001", "3002", "50051"]
  source_ranges      = ["192.168.0.0/24", "10.10.0.0/24"]
  desitnation_ranges = ["0.0.0.0/0"]
  project_id         = var.project_id

  depends_on = [module.Network]
}

module "PublicAccessSubnet" {
  source = "../modules/gcp/network/subnet"

  vpc_id         = module.Network.vpc_id
  subnet_name    = "pub-subnet"
  ip_cidr        = "192.168.0.0/24"
  subnet_purpose = "PRIVATE"
  region         = var.region

  depends_on = [module.Network]
}

module "FirewallRulePublic" {
  source = "../modules/gcp/firewall_rules"

  rule_name          = "public-network-rules"
  vpc_id             = module.Network.vpc_id
  protocol           = "tcp"
  ports              = ["22"]
  source_ranges      = concat(var.ip_isp_pub, ["10.10.0.0/24", "0.0.0.0/0"])
  desitnation_ranges = ["0.0.0.0/0"]
  project_id         = var.project_id

  depends_on = [module.Network]
}




module "BootstrapNode" {
  source = "../modules/gcp/compute/private_vm"

  num_instances      = 1
  vm_name            = "bootstrap-node"
  machine_type       = var.bootstrap_node_machine_type
  vpc_id             = module.Network.vpc_id
  subnet             = local.private_subnet_name
  public_instance    = true
  image              = var.image
  provisioning_model = var.bootstrap_node_provisioning_model
  tags               = var.bootstrap_node_tags
  scopes             = var.scopes
  ssh_pub            = file(var.path_local_public_key)
  username           = var.username
  defaul_sa_name     = data.google_compute_default_service_account.default_sa.email
  available_zones    = ["europe-west4-a", "europe-west4-b", "europe-west4-c"]
  packages           = "protobuf-compiler build-essential gcc make"
  static_ip          = "10.10.0.2"

  depends_on = [module.PrivateAccessSubnet]
}

module "AuctionsServer" {
  source = "../modules/gcp/compute/private_vm"

  num_instances      = var.auctions_server_n_nodes
  vm_name            = "auctions-server"
  machine_type       = var.auctions_server_machine_type
  vpc_id             = module.Network.vpc_id
  subnet             = local.private_subnet_name
  public_instance    = true
  image              = var.image
  provisioning_model = var.auctions_server_provisioning_model
  tags               = var.auctions_server_tags
  scopes             = var.scopes
  ssh_pub            = file(var.path_local_public_key)
  username           = var.username
  defaul_sa_name     = data.google_compute_default_service_account.default_sa.email
  available_zones    = ["europe-west4-a", "europe-west4-b", "europe-west4-c"]
  packages           = "protobuf-compiler build-essential gcc make"

  depends_on = [module.BootstrapNode, module.PrivateAccessSubnet]
}

module "AuctionsClient" {
  source = "../modules/gcp/compute/private_vm"

  num_instances      = var.auction_client_n_nodes
  vm_name            = "auctions-client"
  machine_type       = var.auction_client_machine_type
  vpc_id             = module.Network.vpc_id
  subnet             = local.private_subnet_name
  public_instance    = true
  image              = var.image
  provisioning_model = var.auction_client_provisioning_model
  tags               = var.auction_client_tags
  scopes             = var.scopes
  ssh_pub            = file(var.path_local_public_key)
  username           = var.username
  defaul_sa_name     = data.google_compute_default_service_account.default_sa.email
  available_zones    = ["europe-west4-a", "europe-west4-b", "europe-west4-c"]
  packages           = "protobuf-compiler build-essential gcc make"

  depends_on = [module.BootstrapNode, module.PrivateAccessSubnet]
}
