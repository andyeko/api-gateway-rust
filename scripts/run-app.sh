#!/usr/bin/env bash
set -euo pipefail

FEATURES=${1:-gateway}
WITH_FRONT=${2:-}

if [ "$WITH_FRONT" = "--with-front" ]; then
	if [ -d "front" ]; then
		(cd front && npm install)
		(cd front && npm run dev) &
		echo "Frontend started on http://localhost:5173"
	else
		echo "Front directory not found: front"
	fi
fi

cargo run -p apisentinel-app --features "$FEATURES"
