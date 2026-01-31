#!/usr/bin/env bash
set -euo pipefail

GATEWAY=false
AUTH=false
ADMIN=false
WITH_FRONT=false

if [ "$#" -eq 0 ]; then
  GATEWAY=true
  AUTH=true
  ADMIN=true
else
  for arg in "$@"; do
    case "$arg" in
      --gateway) GATEWAY=true ;;
      --auth) AUTH=true ;;
      --admin) ADMIN=true ;;
      --with-front) WITH_FRONT=true ;;
    esac
  done
fi

$GATEWAY && cargo run -p apisentinel-gateway &
$AUTH && cargo run -p apisentinel-auth &
$ADMIN && cargo run -p apisentinel-admin &

if [ "$WITH_FRONT" = "true" ]; then
  if [ -d "front" ]; then
    (cd front && npm install)
    (cd front && npm run dev) &
    echo "Frontend started on http://localhost:5173"
  else
    echo "Front directory not found: front"
  fi
fi

wait
