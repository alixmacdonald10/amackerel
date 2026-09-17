resource "digitalocean_project" "amackerel" {
  name        = local.project_name
  description = "Personal Website"
  purpose     = "Web Application"
  environment = "Production"
  resources = [
    digitalocean_droplet.amackerel.urn
  ]
}

resource "digitalocean_ssh_key" "amackerel" {
  name       = "${local.project_name}-prod"
  public_key = var.ssh_public_key
}

resource "digitalocean_droplet" "amackerel" {
  image      = "ubuntu-24-04-x64"
  name       = "${local.project_name}-prod"
  region     = "lon1"
  size       = "s-1vcpu-512mb-10gb"
  ssh_keys   = [digitalocean_ssh_key.amackerel.fingerprint]
  backups    = false
  monitoring = true
  ipv6       = false
  tags       = [local.project_name, "prod"]

  user_data = templatefile("${path.module}/cloud-init.yaml.tftpl", {
    image            = var.image
    cf_tunnel_token  = data.cloudflare_zero_trust_tunnel_cloudflared_token.amackerel.token
    github_api_token = var.gh_api_token
  })
}

#trivy:ignore:DIG-0003 -- outbound-only: HTTP/HTTPS/DNS/cloudflared tunnel egress requires unrestricted destination IPs (Cloudflare edge + arbitrary DNS resolvers aren't pinnable to a fixed CIDR)
resource "digitalocean_firewall" "amackerel" {
  name        = "${local.project_name}-waf"
  droplet_ids = [digitalocean_droplet.amackerel.id]
  tags        = [local.project_name, "prod"]

  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = var.ssh_allowed_cidrs
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "80"
    destination_addresses = ["0.0.0.0/0"]
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "443"
    destination_addresses = ["0.0.0.0/0"]
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "53"
    destination_addresses = ["0.0.0.0/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "53"
    destination_addresses = ["0.0.0.0/0"]
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "7844"
    destination_addresses = ["0.0.0.0/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "7844"
    destination_addresses = ["0.0.0.0/0"]
  }

}
