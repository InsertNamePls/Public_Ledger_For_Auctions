#!/bin/bash

# Define the network interface and the delay
INTERFACE="lo"
DELAY="50ms"

# Function to add latency
add_latency() {
    echo "Adding $DELAY latency to $INTERFACE"
    sudo tc qdisc add dev $INTERFACE root netem delay $DELAY
}

# Function to remove latency
remove_latency() {
    echo "Removing latency from $INTERFACE"
    sudo tc qdisc del dev $INTERFACE root netem
}

# Adding latency
add_latency

# Wait for user input to remove latency
read -p "Press Enter to remove latency and exit..."

# Removing latency
remove_latency

echo "Latency removed. Exiting..."
