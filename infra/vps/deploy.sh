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

# 2.5 Pengecekan Dependensi yang Sudah Terinstal (True/False)
echo -e "\n${BLUE}=== Memeriksa Status Instalasi Dependensi ===${NC}"
check_status() {
  if command -v "$1" &> /dev/null; then
    echo -e "  - $2: ${GREEN}True (Sudah terinstal)${NC}"
  else
    echo -e "  - $2: ${RED}False (Belum terinstal)${NC}"
  fi
}

check_status "nginx" "Nginx Reverse Proxy"
check_status "psql" "PostgreSQL Database"
check_status "redis-server" "Redis Cache Server"
check_status "bun" "Bun Runtime"
check_status "cargo" "Rust/Cargo Compiler"
echo -e "=============================================\n"

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
systemctl start postgresql || true
systemctl enable postgresql || true

echo -n "Apakah Anda ingin menggunakan PostgreSQL yang sudah ada dengan kredensial kustom? (y/n) [default: n]: "
read USE_EXISTING_DB

if [ "$USE_EXISTING_DB" = "y" ] || [ "$USE_EXISTING_DB" = "Y" ]; then
  echo -n "Masukkan Host PostgreSQL [default: localhost]: "
  read DB_HOST
  DB_HOST=${DB_HOST:-localhost}

  echo -n "Masukkan Port PostgreSQL [default: 5432]: "
  read DB_PORT
  DB_PORT=${DB_PORT:-5432}

  echo -n "Masukkan Nama Database [default: tiketku]: "
  read DB_NAME
  DB_NAME=${DB_NAME:-tiketku}

  echo -n "Masukkan Username PostgreSQL [default: tiketku]: "
  read DB_USER
  DB_USER=${DB_USER:-tiketku}

  echo -n "Masukkan Password PostgreSQL: "
  read -s DB_PASS
  echo "" # Newline setelah input password tersembunyi
else
  # Default setup (Auto create new user & database locally)
  DB_HOST="localhost"
  DB_PORT="5432"
  DB_NAME="tiketku"
  DB_USER="tiketku"
  DB_PASS="tiketku"

  echo -e "\nMembuat user & database default..."
  sudo -u postgres psql -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';" 2>/dev/null || true
  sudo -u postgres psql -c "ALTER USER $DB_USER WITH SUPERUSER;" 2>/dev/null || true
  sudo -u postgres psql -c "CREATE DATABASE $DB_NAME OWNER $DB_USER;" 2>/dev/null || true
fi

# Import skema database awal
echo -e "${BLUE}Menyalin struktur skema awal database...${NC}"
cd ../..
PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f infra/migrations/0001_initial.sql 2>/dev/null || \
sudo -u postgres psql -d "$DB_NAME" -f infra/migrations/0001_initial.sql 2>/dev/null || \
echo -e "${YELLOW}Warning: Gagal mengimpor skema awal secara langsung. Jika tabel sudah ada, silakan abaikan.${NC}"
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
  if [ -f "backend/.env.example" ]; then
    cp backend/.env.example backend/.env
  else
    echo -e "${YELLOW}Warning: backend/.env.example tidak ditemukan. Membuat file backend/.env baru.${NC}"
    touch backend/.env
  fi
fi

# Update DATABASE_URL di backend/.env secara aman tanpa sed -i
echo -e "${BLUE}Mengonfigurasi DATABASE_URL di backend/.env...${NC}"
if grep -q "^DATABASE_URL=" "backend/.env"; then
  grep -v "^DATABASE_URL=" "backend/.env" > "backend/.env.tmp"
  echo "DATABASE_URL=postgres://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME" >> "backend/.env.tmp"
  mv "backend/.env.tmp" "backend/.env"
else
  echo "DATABASE_URL=postgres://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME" >> "backend/.env"
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
