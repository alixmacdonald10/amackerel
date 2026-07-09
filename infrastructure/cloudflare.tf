# Tunnel secret must be base64-encoded; random_id.b64_std guarantees that.
resource "random_id" "tunnel_secret" {
  byte_length = 32
}

resource "cloudflare_zero_trust_tunnel_cloudflared" "amackerel" {
  account_id    = var.cloudflare_account_id
  name          = "${local.project_name}-prod-do"
  config_src    = "cloudflare"
  tunnel_secret = random_id.tunnel_secret.b64_std
}

# Connector token for cloudflared --token on the droplet.
data "cloudflare_zero_trust_tunnel_cloudflared_token" "amackerel" {
  account_id = var.cloudflare_account_id
  tunnel_id  = cloudflare_zero_trust_tunnel_cloudflared.amackerel.id
}

resource "cloudflare_zero_trust_tunnel_cloudflared_config" "amackerel" {
  account_id = var.cloudflare_account_id
  tunnel_id  = cloudflare_zero_trust_tunnel_cloudflared.amackerel.id

  config = {
    ingress = [
      {
        hostname = "${local.project_name}.dev"
        service  = "http://localhost:8080"
      },
      # Catch-all required by Cloudflare: last rule must match all URLs.
      {
        service = "http_status:404"
      },
    ]
  }
}

# Route amackerel.dev through the tunnel via proxied CNAME.
resource "cloudflare_dns_record" "amackerel" {
  zone_id = var.cloudflare_dns_zone_id
  name    = "${local.project_name}.dev"
  type    = "CNAME"
  content = "${cloudflare_zero_trust_tunnel_cloudflared.amackerel.id}.cfargotunnel.com"
  proxied = true
  ttl     = 1
}
