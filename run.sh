#!/bin/bash
for dir in */; do
  if [ -f "$dir/Cargo.toml" ]; then
    echo "Building $dir"
    (cd "$dir" && cargo build)
  fi
done


# Start the server container in detached mode
docker compose up

# Wait for the server to start
sleep 2

# Start the client container in detached mode
docker run -d --name client_container client

