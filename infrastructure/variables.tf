# Set the variable value in *.tfvars file
# or using -var="do_token=..." CLI option
variable "do_token" {
  type      = string
  sensitive = true
}

variable "cloudflare_api_token" {
  type      = string
  sensitive = true
}

variable "cloudflare_account_id" {
  type      = string
  sensitive = true
}

variable "cloudflare_dns_zone_id" {
  type      = string
  sensitive = true
}

# Container image to run. Defaults to the latest published amackerel image.
variable "image" {
  default = "ghcr.io/alixmacdonald10/amackerel:latest"
}

variable "gh_api_token" {
  type      = string
  sensitive = true
}

variable "ssh_allowed_cidrs" {
  type = list(string)
}

# Public key content, not a path — portable across local runs and CI, where
# no `~/.ssh` of yours exists. Not sensitive: it's a public key.
variable "ssh_public_key" {
  type = string
}
