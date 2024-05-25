variable "rule_name" {
    type = string
}
variable "vpc_id" {
    type = string
}
variable "protocol" {
    type = string
}

variable "ports" {
    type = list(string)
}
variable "project_id" {
  type = string
}
variable "source_ranges" {
  type = list(string)
}
variable "desitnation_ranges" {
  type = list(string)
}