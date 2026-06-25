#!/bin/bash
# ============================================================
# TiketKu POS & Ticketing System — VPS Production Deployment Script
# ============================================================

set -e

# Setup colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Memulai TiketKu Production VPS Deployment ===${NC}"

# 1. Pastikan dijalankan sebagai root
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Error: Script ini harus dijalankan dengan hak akses root / sudo.${NC}"
  exit 1
fi

# 2. Input Domain/IP
echo -n "Masukkan Domain VPS Anda (contoh: tiketku.co.id): "
read DOMAIN
if [ -z "$DOMAIN" ]; then
  echo -e "${RED}Error: Domain/IP wajib diisi.${NC}"
  exit 1
fi

# 3. Install System Dependencies (Ubuntu/Debian)
echo -e "\n${BLUE}[1/8] Menginstal paket-paket dasar Linux...${NC}"
apt-get update
apt-get install -y curl wget git build-essential pkg-config libssl-dev lsof nginx redis-server postgresql postgresql-contrib certbot python3-certbot-nginx

# 4. Install Bun (Jika belum terpasang)
if ! command -v bun &> /dev/null; then
  echo -e "\n${BLUE}[2/8] Menginstal Bun Runtime...${NC}"
  curl -fsSL https://bun.sh/install | bash
  # Load Bun binary to current shell session
  export BUN_INSTALL="$HOME/.bun"
  export PATH="$BUN_INSTALL/bin:$PATH"
else
  echo -e "\n${GREEN}Bun Runtime sudah terinstal.${NC}"
fi

# 5. Install Rust & Cargo (Jika belum terpasang)
if ! command -v cargo &> /dev/null; then
  echo -e "\n${BLUE}[3/8] Menginstal Rust Compiler...${NC}"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
else
  echo -e "\n${GREEN}Rust Compiler sudah terinstal.${NC}"
fi

# 6. Setup PostgreSQL Database
echo -e "\n${BLUE}[4/8] Mengonfigurasi PostgreSQL...${NC}"
# Pastikan Postgres aktif
systemctl start postgresql
systemctl enable postgresql

# Buat role tiketku dan database tiketku jika belum ada
sudo -u postgres psql -c "CREATE USER tiketku WITH PASSWORD 'tiketku';" 2>/dev/null || true
sudo -u postgres psql -c "ALTER USER tiketku WITH SUPERUSER;" 2>/dev/null || true
sudo -u postgres psql -c "CREATE DATABASE tiketku OWNER tiketku;" 2>/dev/null || true

# Import skema database awal
echo -e "${BLUE}Menyalin struktur skema awal database...${NC}"
cd ../..
sudo -u postgres psql -d tiketku -f infra/migrations/0001_initial.sql
cd infra/vps

# 7. Salin & Siapkan Environment (.env)
echo -e "\n${BLUE}[5/8] Mengonfigurasi Environment File...${NC}"
# Buat direktori /var/www jika belum ada, salin source code ke sana
APP_DIR="/var/www/e-ticketing"
if [ "$(pwd)" != "$APP_DIR" ]; then
  echo -e "${YELLOW}Menyalin source code ke $APP_DIR...${NC}"
  mkdir -p $APP_DIR
  cp -r ../../* $APP_DIR/
  cd $APP_DIR
fi

# Generate .env untuk backend jika belum ada
if [ ! -f "backend/.env" ]; then
  cp backend/.env.example backend/.env 2>/dev/null || cp backend/.env backend/.env
fi

# Update ALLOWED_ORIGINS di backend/.env
existing_origins=$(grep "^ALLOWED_ORIGINS=" "backend/.env" | cut -d'=' -f2-)
if [[ ! "$existing_origins" =~ "$DOMAIN" ]]; then
  new_origins="${existing_origins},https://${DOMAIN},http://${DOMAIN}"
  grep -v "^ALLOWED_ORIGINS=" "backend/.env" > "backend/.env.tmp"
  echo "ALLOWED_ORIGINS=${new_origins}" >> "backend/.env.tmp"
  mv "backend/.env.tmp" "backend/.env"
fi

# 8. Build Backend (Rust Release Build)
echo -e "\n${BLUE}[6/8] Membangun Backend Rust (Mode Release)...${NC}"
cd backend
cargo build --release
cd ..

# 9. Build Frontend (SvelteKit Production Build)
echo -e "\n${BLUE}[7/8] Menginstal Dependensi & Membangun Frontend SvelteKit...${NC}"
cd frontend
# Pastikan bun terbaca
export BUN_INSTALL="$HOME/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"
bun install
bun run build
cd ..

# 10. Konfigurasi Systemd Service Backend
echo -e "\n${BLUE}[8/8] Memasang Systemd Daemon Service Backend...${NC}"
# Copy backend service
cp infra/vps/tiketku-backend.service /etc/systemd/system/tiketku-backend.service

# Reload systemd and enable/restart backend service
systemctl daemon-reload
systemctl enable tiketku-backend
systemctl restart tiketku-backend

# 11. Konfigurasi Nginx
echo -e "\n${BLUE}Mengonfigurasi Nginx Reverse Proxy...${NC}"
sed "s/YOUR_DOMAIN_OR_IP/$DOMAIN/g" infra/vps/nginx.conf > /etc/nginx/sites-available/tiketku
ln -sf /etc/nginx/sites-available/tiketku /etc/nginx/sites-enabled/default
systemctl restart nginx

# 12. Pilihan SSL Otomatis Certbot (Let's Encrypt)
echo -e "\n${GREEN}=== Deployment Sukses! ===${NC}"
echo -e "Apakah Anda ingin memasang SSL gratis Let's Encrypt secara otomatis sekarang?"
echo -e "  [1] Ya, daftarkan sertifikat SSL Certbot untuk ${DOMAIN}"
echo -e "  [2] Tidak, biarkan menggunakan konfigurasi default"
echo -n "Pilihan Anda (1/2): "
read ssl_choice
if [ "$ssl_choice" = "1" ]; then
  echo -e "\n${BLUE}Menjalankan Certbot SSL...${NC}"
  certbot --nginx -d "$DOMAIN" -d "www.$DOMAIN" --non-interactive --agree-tos --redirect -m admin@"$DOMAIN"
  systemctl restart nginx
  echo -e "${GREEN}✔ SSL Berhasil Terpasang!${NC}"
fi

echo -e "\n${GREEN}=== TiketKu System Telah Aktif di VPS! ===${NC}"
echo -e "Buka aplikasi di browser: ${BLUE}https://${DOMAIN}${NC}"
echo -e "Log API Backend: ${YELLOW}journalctl -u tiketku-backend -f${NC}"
echo -e "Frontend disajikan secara statis oleh Nginx dari: ${YELLOW}/var/www/e-ticketing/frontend/build${NC}"
