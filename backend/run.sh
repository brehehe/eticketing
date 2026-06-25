#!/bin/zsh
# Kill any existing process on port 8080
printf 'Mematikan proses lama di port 8080...\n'
lsof -ti:8080 | xargs kill -9 2>/dev/null
sleep 1

printf 'Menjalankan backend...\n'
exec cargo run
