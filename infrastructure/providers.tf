terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5"
    }
  }

  # State stored in Cloudflare R2 (S3-compatible).
  # Credentials via r2.backend.hcl.
  backend "s3" {
    bucket = "amackerel-iac"
    key    = "amackerel-prod.terraform.tfstate"
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

provider "cloudflare" {
  api_token = var.cloudflare_api_token
}

provider "digitalocean" {
  token = var.do_token
}
