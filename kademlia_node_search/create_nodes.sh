#!/bin/bash

# Define the base port number
base_port=50050
# Define the first node's address
bootstrap_addr="127.0.0.1:$((base_port+1))"

# Create a directory for logs if it doesn't exist
mkdir -p kademlia_logs

# Start the first node without a bootstrap node
echo "Starting node 1 at port $((base_port+1))"
RUST_LOG=info cargo run -- server 127.0.0.1:$((base_port+1)) > kademlia_logs/node_1.log 2>&1 &

# Wait a bit to ensure the first node starts up properly
sleep 5

# Start the remaining nodes with the first node as the bootstrap node
for i in {2..40}
do
  echo "Starting node $i at port $((base_port+i))"
  RUST_LOG=info cargo run -- server 127.0.0.1:$((base_port+i)) --bootstrap $bootstrap_addr > kademlia_logs/node_$i.log 2>&1 &
  sleep 2  # Optional: to stagger the start times slightly
done

echo "All nodes started. Logs are being written to kademlia_logs/"
