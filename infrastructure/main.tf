terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

# Set the variable value in *.tfvars file
# or using -var="do_token=..." CLI option
variable "do_token" {}

# Cloudflare tunnel token for the named tunnel. Ingress (hostname -> localhost:8080)
# is configured dashboard-side. Keep this out of source control (tfvars / -var / env).
variable "cf_tunnel_token" {
  sensitive = true
}

# Container image to run. Defaults to the latest published amackerel image.
variable "image" {
  default = "ghcr.io/alixmacdonald10/amackerel:latest"
}

# Configure the DigitalOcean Provider
provider "digitalocean" {
  token = var.do_token
}

# Create a new SSH key
resource "digitalocean_ssh_key" "amackerel" {
  name       = "amackerel-prod"
  public_key = file("/home/utsar/.ssh/id_ed25519_do_amackerel.pub")
}

# Create a new Web Droplet in the lon1 region
resource "digitalocean_droplet" "amackerel" {
  image      = "ubuntu-24-04-x64"
  name       = "amackerel-prod"
  region     = "lon1"
  size       = "s-1vcpu-512mb-10gb"
  ssh_keys   = [digitalocean_ssh_key.amackerel.fingerprint]
  backups    = false
  monitoring = true
  ipv6       = false
  tags       = ["amackerel", "prod"]

  user_data = templatefile("${path.module}/cloud-init.yaml.tftpl", {
    image           = var.image
    cf_tunnel_token = var.cf_tunnel_token
  })
}
