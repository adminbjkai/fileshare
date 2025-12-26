# FileShare Deployment Guide

## Quick Deploy to Linux Server

### 1. Copy Files to Server

```bash
# On your local machine
scp -r target/release/fileshare static/ user@server:/opt/fileshare/
```

Or create the directory structure on the server:

```bash
# On the server
sudo mkdir -p /opt/fileshare/{uploads,static}
sudo chown -R www-data:www-data /opt/fileshare
sudo chmod 755 /opt/fileshare/static
sudo chmod 775 /opt/fileshare/uploads
```

### 2. Copy the Binary and Static Files

```bash
# Copy binary
sudo cp fileshare /opt/fileshare/

# Copy static files
sudo cp -r static/* /opt/fileshare/static/

# Set permissions
sudo chmod +x /opt/fileshare/fileshare
sudo chown -R www-data:www-data /opt/fileshare
sudo find /opt/fileshare/static -type f -exec chmod 644 {} +
sudo find /opt/fileshare/uploads -type f -exec chmod 664 {} +
```

### 3. Install systemd Service

```bash
# Copy service file
sudo cp docs/fileshare.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable and start service
sudo systemctl enable fileshare
sudo systemctl start fileshare

# Check status
sudo systemctl status fileshare
```

### 4. Configure Nginx

```bash
# Copy nginx config
sudo cp docs/nginx-fileshare.conf /etc/nginx/sites-available/fileshare

# Enable site
sudo ln -s /etc/nginx/sites-available/fileshare /etc/nginx/sites-enabled/

# Test config
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx
```

### 5. SSL Certificate (Let's Encrypt)

```bash
# Install certbot if not installed
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d fileshare.bjk.ai

# Certbot will auto-configure nginx
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `FILESHARE_HOST` | `0.0.0.0` | Bind address |
| `FILESHARE_PORT` | `7012` | Server port |
| `FILESHARE_BASE_URL` | `http://localhost:7012` | Public URL for QR codes |

## Directory Structure on Server

```
/opt/fileshare/
├── fileshare          # Binary
├── static/
│   └── index.html     # Frontend
└── uploads/           # Uploaded files
    └── .zipcount      # Auto-increment counter
```

## Logs

```bash
# View service logs
sudo journalctl -u fileshare -f

# View nginx access logs
sudo tail -f /var/log/nginx/access.log
```

## Maintenance

```bash
# Restart service
sudo systemctl restart fileshare

# Stop service
sudo systemctl stop fileshare

# Clear uploads (if needed)
sudo rm -rf /opt/fileshare/uploads/*
sudo touch /opt/fileshare/uploads/.zipcount
echo "0" | sudo tee /opt/fileshare/uploads/.zipcount
```
