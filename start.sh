#!/bin/zsh

# exit on error
set -e

# Setup colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== TiketKu POS & Ticketing System Launcher ===${NC}"

# Trap Ctrl+C (SIGINT) to kill all background processes
cleanup() {
  echo -e "\n${YELLOW}Menghentikan semua proses dev server...${NC}"
  pkill -P $$ 2>/dev/null || true
  echo -e "${GREEN}Selesai. Sampai jumpa!${NC}"
  exit 0
}
trap cleanup SIGINT

# Default variables
MODE=""
DOMAIN=""
DB_CHOICE=""
SSL_CHOICE=""
export SSL_KEY_PATH=""
export SSL_CERT_PATH=""

# Parse arguments
while [[ "$#" -gt 0 ]]; do
  case $1 in
    local) MODE="local"; shift ;;
    vps) MODE="vps"; shift ;;
    --domain) DOMAIN="$2"; shift 2 ;;
    --db) DB_CHOICE="$2"; shift 2 ;; # "local" or "docker"
    --ssl-key) export SSL_KEY_PATH="$2"; shift 2 ;;
    --ssl-cert) export SSL_CERT_PATH="$2"; shift 2 ;;
    *) echo "Unknown parameter: $1"; exit 1 ;;
  esac
done

update_allowed_origins() {
  local domain=$1
  local env_file="backend/.env"
  if [ -f "$env_file" ]; then
    local existing=$(grep "^ALLOWED_ORIGINS=" "$env_file" | cut -d'=' -f2-)
    if [[ ! "$existing" =~ "$domain" ]]; then
      local new_origins="${existing},https://${domain},https://${domain}:5173,https://${domain}:5174,http://${domain},http://${domain}:5173,http://${domain}:5174"
      grep -v "^ALLOWED_ORIGINS=" "$env_file" > "${env_file}.tmp"
      echo "ALLOWED_ORIGINS=${new_origins}" >> "${env_file}.tmp"
      mv "${env_file}.tmp" "$env_file"
      echo -e "${GREEN}✔ Berhasil menambahkan domain ${domain} ke ALLOWED_ORIGINS.${NC}"
    fi
  fi
}

# Interactive Menu if no mode selected
if [ -z "$MODE" ]; then
  echo -e "Pilih mode eksekusi:"
  echo -e "  [1] ${GREEN}Lokal${NC} (Menggunakan PostgreSQL & Redis lokal Homebrew/systemd)"
  echo -e "  [2] ${YELLOW}VPS Server${NC} (Mendukung custom domain, SSL asli, atau Docker)"
  echo -n "Pilihan Anda (1/2): "
  read choice
  if [ "$choice" = "1" ]; then
    MODE="local"
  elif [ "$choice" = "2" ]; then
    MODE="vps"
  else
    echo -e "${RED}Pilihan tidak valid!${NC}"
    exit 1
  fi
fi

if [ "$MODE" = "local" ]; then
  echo -e "\n${BLUE}--- Memulai Mode Lokal ---${NC}"
  
  # 1. Pastikan PostgreSQL lokal berjalan
  echo -e "${BLUE}[1/4] Memeriksa PostgreSQL lokal...${NC}"
  if ! pg_isready -h localhost -p 5432 -q; then
    echo -e "${YELLOW}PostgreSQL lokal tidak aktif. Mencoba mengaktifkan...${NC}"
    if command -v brew &> /dev/null; then
      brew services start postgresql@17 || brew services start postgresql
    else
      sudo systemctl start postgresql || sudo service postgresql start
    fi
    sleep 2
    if ! pg_isready -h localhost -p 5432 -q; then
      echo -e "${RED}Error: Gagal menjalankan PostgreSQL lokal. Pastikan database aktif.${NC}"
      exit 1
    fi
  fi
  echo -e "${GREEN}PostgreSQL lokal aktif.${NC}"

  # 2. Pastikan Redis lokal berjalan
  echo -e "${BLUE}[2/4] Memeriksa Redis lokal...${NC}"
  if ! redis-cli ping >/dev/null 2>&1; then
    echo -e "${YELLOW}Redis lokal tidak aktif. Mencoba mengaktifkan...${NC}"
    if command -v brew &> /dev/null; then
      brew services start redis
    else
      sudo systemctl start redis || sudo service redis-server start
    fi
    sleep 2
    if ! redis-cli ping >/dev/null 2>&1; then
      echo -e "${RED}Error: Gagal menjalankan Redis lokal. Pastikan redis-server aktif.${NC}"
      exit 1
    fi
  fi
  echo -e "${GREEN}Redis lokal aktif.${NC}"

else
  # VPS Mode
  echo -e "\n${BLUE}--- Memulai Mode VPS Server ---${NC}"
  
  if [ -z "$DOMAIN" ]; then
    echo -n "Masukkan Domain atau IP Publik VPS Anda (contoh: tiketku.co.id): "
    read DOMAIN
    if [ -z "$DOMAIN" ]; then
      echo -e "${RED}Domain/IP wajib diisi untuk mode VPS!${NC}"
      exit 1
    fi
  fi

  # Update backend allowed origins
  update_allowed_origins "$DOMAIN"

  # Database/Redis choice
  if [ -z "$DB_CHOICE" ]; then
    echo -e "\nPilih metode Database & Redis:"
    echo -e "  [1] ${GREEN}Lokal Service${NC} (Homebrew/systemd service yang berjalan di VPS)"
    echo -e "  [2] ${YELLOW}Docker Compose${NC} (PostgreSQL & Redis di dalam Docker)"
    echo -n "Pilihan Anda (1/2): "
    read db_choice_input
    if [ "$db_choice_input" = "1" ]; then
      DB_CHOICE="local"
    elif [ "$db_choice_input" = "2" ]; then
      DB_CHOICE="docker"
    else
      echo -e "${RED}Pilihan tidak valid! Default ke Lokal Service.${NC}"
      DB_CHOICE="local"
    fi
  fi

  if [ "$DB_CHOICE" = "docker" ]; then
    echo -e "${BLUE}[1/4] Menjalankan Database & Cache via Docker...${NC}"
    docker compose -f infra/docker/docker-compose.yml up -d postgres redis
  else
    echo -e "${BLUE}[1/4] Memeriksa Database & Cache lokal VPS...${NC}"
    # Check Postgres
    if ! pg_isready -h localhost -p 5432 -q; then
      echo -e "${YELLOW}Mencoba menghidupkan PostgreSQL lokal...${NC}"
      if command -v brew &> /dev/null; then
        brew services start postgresql@17 || brew services start postgresql
      else
        sudo systemctl start postgresql || sudo service postgresql start
      fi
      sleep 2
    fi
    # Check Redis
    if ! redis-cli ping >/dev/null 2>&1; then
      echo -e "${YELLOW}Mencoba menghidupkan Redis lokal...${NC}"
      if command -v brew &> /dev/null; then
        brew services start redis
      else
        sudo systemctl start redis || sudo service redis-server start
      fi
      sleep 2
    fi
  fi

  # SSL Custom Choice
  if [ -z "$SSL_KEY_PATH" ] && [ -z "$SSL_CERT_PATH" ]; then
    echo -e "\nApakah Anda memiliki sertifikat SSL asli untuk domain $DOMAIN?"
    echo -e "  [1] ${GREEN}Ya, saya ingin menggunakan SSL asli saya (Let's Encrypt / Custom PEM)${NC}"
    echo -e "  [2] ${YELLOW}Tidak, gunakan SSL self-signed otomatis${NC}"
    echo -n "Pilihan Anda (1/2): "
    read ssl_choice_input
    if [ "$ssl_choice_input" = "1" ]; then
      echo -n "Masukkan path file SSL Private Key (.key): "
      read key_path
      echo -n "Masukkan path file SSL Certificate (.crt/.pem): "
      read cert_path
      export SSL_KEY_PATH="$key_path"
      export SSL_CERT_PATH="$cert_path"
    fi
  fi

  if [ -n "$SSL_KEY_PATH" ] && [ -n "$SSL_CERT_PATH" ]; then
    echo -e "${GREEN}✔ Menggunakan sertifikat SSL kustom untuk Vite.${NC}"
  else
    echo -e "${YELLOW}⚠ Menggunakan SSL self-signed otomatis.${NC}"
  fi
fi

# 3. Jalankan Backend (Rust)
echo -e "${BLUE}[3/4] Menjalankan Rust Backend (Port 8080)...${NC}"
cd backend
lsof -ti:8080 | xargs kill -9 2>/dev/null || true
cargo run > backend.log 2>&1 &
BACKEND_PID=$!
cd ..

# 4. Jalankan Frontend (SvelteKit + Bun)
echo -e "${BLUE}[4/4] Menjalankan SvelteKit Frontend (Port 5173/5174)...${NC}"
cd frontend
lsof -ti:5173 | xargs kill -9 2>/dev/null || true
lsof -ti:5174 | xargs kill -9 2>/dev/null || true

# Run Dev Server in background
bun run dev --host > frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..

# 5. Jalankan Local Print Bridge (Port 9100)
if [ "$MODE" = "local" ]; then
  echo -e "${BLUE}[Bonus] Menjalankan Local Print Bridge (Port 9100)...${NC}"
  lsof -ti:9100 | xargs kill -9 2>/dev/null || true
  node printbridge.js > printbridge.log 2>&1 &
  BRIDGE_PID=$!
fi

echo -e "${GREEN}=== Semua Service Berhasil Dijalankan! ===${NC}"
echo -e "Silakan buka browser:"
if [ "$MODE" = "local" ]; then
  echo -e "  - Frontend (HTTPS): ${BLUE}https://localhost:5173${NC} atau ${BLUE}https://192.168.111.250:5173${NC}"
  echo -e "  - Print Bridge:     ${BLUE}http://localhost:9100/status${NC}"
else
  echo -e "  - Frontend (HTTPS): ${BLUE}https://${DOMAIN}:5173${NC}"
fi
echo -e "  - Backend API: ${BLUE}http://localhost:8080/api/health${NC}"
echo -e "\nLog backend dapat dilihat di: ${YELLOW}backend/backend.log${NC}"
echo -e "Log frontend dapat dilihat di: ${YELLOW}frontend/frontend.log${NC}"
if [ "$MODE" = "local" ]; then
  echo -e "Log print bridge dapat dilihat di: ${YELLOW}printbridge.log${NC}"
fi
echo -e "${YELLOW}Tekan Ctrl+C untuk menghentikan semua service.${NC}"

wait
