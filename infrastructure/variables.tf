# Set the variable value in *.tfvars file
# or using -var="do_token=..." CLI option
variable "do_token" {
  sensitive = true
}

variable "cloudflare_api_token" {
  sensitive = true
}

variable "cloudflare_account_id" {
  sensitive = true
}

variable "cloudflare_dns_zone_id" {
  sensitive = true
}

# Container image to run. Defaults to the latest published amackerel image.
variable "image" {
  default = "ghcr.io/alixmacdonald10/amackerel:latest"
}
