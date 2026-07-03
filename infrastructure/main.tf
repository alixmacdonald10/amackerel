terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }

  # State stored in Cloudflare R2 (S3-compatible).
  # Credentials via r2.backend.hcl.
  backend "s3" {
    bucket = "amackerel-iac"
    key    = "terraform.tfstate"
    region = "auto"

    endpoints = {
      s3 = "https://0a03eb03e9d2b915d12d7e813bca1f9e.r2.cloudflarestorage.com"
    }

    # R2 is not real S3 — skip AWS-specific validation/calls.
    skip_credentials_validation = true
    skip_region_validation      = true
    skip_metadata_api_check     = true
    skip_requesting_account_id  = true
    skip_s3_checksum            = true
    use_path_style              = true
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

resource "digitalocean_firewall" "amackerel" {
  name        = "amackerel-waf"
  droplet_ids = [digitalocean_droplet.amackerel.id]
  tags        = ["amackerel", "prod"]
  # ssh inbound
  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  # all tcp or udp outbound
  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}
