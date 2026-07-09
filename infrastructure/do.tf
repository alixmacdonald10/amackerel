resource "digitalocean_project" "amackerel" {
  name        = local.project_name
  description = "Personal Website"
  purpose     = "Web Application"
  environment = "Production"
  resources = [
    "${digitalocean_droplet.amackerel.urn}"
  ]
}

resource "digitalocean_ssh_key" "amackerel" {
  name       = "${local.project_name}-prod"
  public_key = file("/home/utsar/.ssh/id_ed25519_do_amackerel.pub")
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
    image           = var.image
    cf_tunnel_token = data.cloudflare_zero_trust_tunnel_cloudflared_token.amackerel.token
  })
}

resource "digitalocean_firewall" "amackerel" {
  name        = "${local.project_name}-waf"
  droplet_ids = [digitalocean_droplet.amackerel.id]
  tags        = [local.project_name, "prod"]
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
