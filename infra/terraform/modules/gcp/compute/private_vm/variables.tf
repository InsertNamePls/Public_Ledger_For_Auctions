variable "num_instances" {
  type = number
}
variable "vm_name" {
  type = string
}
variable "machine_type" {
  type = string
}
variable "vpc_id" {
  type = string
}
variable "subnet" {
  type = string
}
variable "image"{
  type = string
}
variable "provisioning_model"{
  type = string
}
variable "tags"{
  type = list(string)
}

variable "scopes"{
  type = list(string)
}
variable "ssh_pub"{
  type = string
  sensitive = true
}

variable "public_instance"{
  type = bool
}
variable "username"{
  type = string
}
variable "defaul_sa_name"{
  type = string
}

variable "available_zones"{
  type = list
}

variable "packages"{
  type = string
}

variable "static_ip"{
  type = string
  default = ""
}