#!/bin/bash

# Kill all processes that match the pattern for our Kademlia nodes
pkill -f 'target/debug/kademlia_node_search'
echo "All Kademlia nodes have been stopped."
