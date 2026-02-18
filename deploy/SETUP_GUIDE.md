# Xonotic RustChain Arena Server Setup Guide

**Bounty #291** | **Reward: 15 RTC** | **Author: admin1douyin**

---

## Overview

This guide covers setting up a complete Xonotic RustChain Arena server with RTC reward system on Linux (Ubuntu/Debian).

## Prerequisites

- Linux server (Ubuntu 20.04+ or Debian 11+)
- Minimum 2 CPU cores, 4GB RAM
- 10GB disk space
- Static IP address
- Open ports: 26000/UDP (game), 80/TCP (web)

## Step 1: System Preparation

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y python3 python3-pip git wget curl nginx certbot

# Create user
sudo useradd -m -s /bin/bash xonotic
sudo usermod -aG sudo xonotic
```

## Step 2: Xonotic Server Installation

```bash
# Switch to xonotic user
sudo su - xonotic

# Create directories
mkdir -p ~/xonotic/{data,logs,config}

# Download Xonotic server
cd ~/xonotic
wget https://dl.xonotic.org/xonotic-0.8.5.zip
unzip xonotic-0.8.5.zip
cd xonotic-0.8.5

# Download full textures
wget https://dl.xonotic.org/xonotic-0.8.5-assets.zip
unzip -o xonotic-0.8.5-assets.zip
```

## Step 3: RustChain Integration

```bash
# Clone RustChain integration
cd ~/xonotic
git clone https://github.com/Scottcjn/xonotic-rustchain.git
cd xonotic-rustchain

# Install Python dependencies
pip3 install -r requirements.txt
```

## Step 4: Server Configuration

Create `~/.xonotic/server.cfg`:

```c
// Xonotic RustChain Arena Server
hostname "Xonotic RustChain Arena"
maxplayers 16
sv_public 1
sv_region 255

// RTC Reward Configuration
sv_rtc_enabled 1
sv_rtc_api_url "https://api.rustchain.org/rewards"
sv_rtc_wallet "YOUR_WALLET_ADDRESS"
```

## Step 5: Firewall Configuration

```bash
# UFW firewall
sudo ufw allow 26000/udp
sudo ufw allow 80/tcp
sudo ufw enable
```

## Step 6: Systemd Service Files

Create `/etc/systemd/system/xonotic.service`:

```ini
[Unit]
Description=Xonotic RustChain Arena Server
After=network.target

[Service]
Type=simple
User=xonotic
WorkingDirectory=/home/xonotic/xonotic-0.8.5
ExecStart=/home/xonotic/xonotic-0.8.5/xonotic-linux64-dedicated +set fs_game rustchain +exec server.cfg
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable xonotic
sudo systemctl start xonotic
```

## Step 7: RustChain Rewards Bridge

Create `/etc/systemd/system/rustchain-bridge.service`:

```ini
[Unit]
Description=RustChain Rewards Bridge
After=network.target xonotic.service

[Service]
Type=simple
User=xonotic
WorkingDirectory=/home/xonotic/xonotic-rustchain
ExecStart=/usr/bin/python3 rustchain_rewards_bridge.py
Restart=always

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable rustchain-bridge
sudo systemctl start rustchain-bridge
```

## Step 8: Nginx Reverse Proxy

Create `/etc/nginx/sites-available/xonotic`:

```nginx
server {
    listen 80;
    server_name your-server.com;

    location / {
        proxy_pass http://127.0.0.1:26000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

Enable and configure SSL:

```bash
sudo ln -s /etc/nginx/sites-available/xonotic /etc/nginx/sites-enabled/
sudo certbot --nginx -d your-server.com
```

## Performance Tuning

Add to `/etc/sysctl.conf`:

```bash
# Increase max connections
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535

# Apply changes
sudo sysctl -p
```

## Verification

Check services:

```bash
sudo systemctl status xonotic
sudo systemctl status rustchain-bridge
sudo journalctl -u xonotic -f
```

Test API:

```bash
curl http://localhost:26000/status
```

## Troubleshooting

### Server won't start
- Check logs: `journalctl -u xonotic`
- Verify port not in use: `sudo netstat -tulpn | grep 26000`

### No rewards
- Check bridge logs: `journalctl -u rustchain-bridge -f`
- Verify wallet address in config

### Performance issues
- Reduce maxplayers in server.cfg
- Check server resources: `htop`

---

**Submit:** Fork https://github.com/Scottcjn/xonotic-rustchain, add files to `deploy/`, open PR.

**Bonus (10 RTC):** CI/CD pipeline + backup script
