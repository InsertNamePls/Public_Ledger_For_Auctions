#!/bin/bash

# Function to remove existing Docker containers and images to start clean
cleanup_docker() {
    docker stop test1 &>/dev/null
    docker rm test1 &>/dev/null
    docker rmi $(docker images | grep 'dledger2auction' | awk '{print $3}') &>/dev/null
}

# Build the Docker image
echo "Building Docker image..."
docker build . --tag dledger2auction

# Create Docker instance for auction app
echo "Creating Docker instance for the auction app..."
docker run --name=test1 -dit dledger2auction

# Prepare the auction app (commands executed within the Docker container)
echo "Preparing the auction app..."
docker exec test1 sh -c "cd home/auction_app && cargo build"
# Note: Automating `cargo run` with user interaction for registration is complex and might need manual intervention

# Prepare instance for the miner (test 1)
echo "Preparing instance for the miner (test 1)..."
docker exec test1 sh -c "cd home/public_ledger && cargo build && cargo run --bin blockchain_operator -- init_blockchain 172.17.0.3" &

# Prepare instance for the miner (test 2)
echo "Preparing instance for the miner (test 2)..."
docker exec test1 sh -c "cd home/public_ledger && cargo build && cargo run --bin blockchain_operator -- init_blockchain 172.17.0.2" &

# Instructions for manual steps
echo "Please manually execute 'cargo run' inside '/home/auction_app' within the 'test1' container to complete the auction app setup, including user registration."
echo "This can be done by running: docker exec -it test1 sh -c 'cd home/auction_app && cargo run'"

# Cleanup function call (optional, uncomment if needed)
# cleanup_docker

echo "Script execution complete. Manual steps may be required to finalize the setup."
