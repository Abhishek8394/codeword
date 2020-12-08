#! /bin/bash
set -euo pipefail

lid=`curl -X POST "http://127.0.0.1:8080/lobby"`
for num in {1..5}; do
	player="player-${num}"
	echo "Creating player:$player"
	curl -X POST "http://127.0.0.1:8080/lobby/${lid}/players" -H "Content-Type: application/json" -d "{\"name\": \"$player\", \"id\": $num}"
	echo
done
