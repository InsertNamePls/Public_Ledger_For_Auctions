terraform {
  backend "gcs" {
    bucket  = "auctions_pub_ledger_tfstate"
    encrypt = true
  }
}
