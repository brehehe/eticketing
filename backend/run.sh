#!/bin/zsh
# Kill any existing process on port 8081
printf 'Mematikan proses lama di port 8081...\n'
lsof -ti:8081 | xargs kill -9 2>/dev/null
sleep 1

printf 'Menjalankan backend...\n'
exec cargo run
