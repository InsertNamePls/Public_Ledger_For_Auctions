
tfstate_bucket_name="auctions_pub_ledger_tfstate"
service_account_file="~/fcupprojects2324-0be80faf5de6.json"
project_name="fcupProjects2324"
project_id="fcupprojects2324"
region="europe-west4"
ip_isp_pub=["149.90.112.98/32"]
path_local_public_key="../../ssh_keys/idrsa.pub"
username="auctions_user"
image="debian-cloud/debian-11"
scopes=["cloud-platform"]

auctions_server_n_nodes="1"
auctions_server_machine_type="e2-small"
auctions_server_provisioning_model="SPOT"
auctions_server_tags=["ssh", "auctions-server"]

bootstrap_node_n_nodes="1"
bootstrap_node_machine_type="e2-small"
bootstrap_node_provisioning_model="SPOT"
bootstrap_node_tags=["ssh", "bootstrap-node"]

auction_client_n_nodes="1"
auction_client_machine_type="e2-small"
auction_client_provisioning_model="SPOT"
auction_client_tags=["ssh", "auction-client"]

