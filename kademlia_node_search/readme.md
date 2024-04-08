# Kademlia DHT Node in Rust
This component implements a basic version of a Kademlia Distributed Hash Table (DHT) node using Rust and gRPC. Kademlia is a decentralized peer-to-peer network protocol that enables nodes in the network to store and retrieve data in a distributed manner. This implementation focuses on the core functionalities of the Kademlia protocol, including node discovery, data storage, and retrieval.

## Features
- **PING**: Probes a node to check its online status.
- **STORE**: Instructs a node to store a key-value pair.
- **FIND_NODE**: Returns information about the k closest nodes to a given target node ID.
- **FIND_VALUE**: Similar to FIND_NODE, but returns the stored value for a given key if available.

## Prerequisites
Before you begin, ensure you have met the following requirements:

- Rust and Cargo installed on your machine.
- gRPC and Protocol Buffers setup for Rust. tonic is used for gRPC implementation.

## Installation

1. Clone the repository:
```bash
git clone https://your-repository-url/kademlia-dht-rust.git
cd kademlia-dht-rust
```
2. Build the project:
```bash
cargo build
```
This command compiles the project and generates executable binaries for the server and client.

## Usage
To run a Kademlia node as a server:

```bash
cargo run -- server <IP:PORT>
```
Replace **\<IP:PORT\>** with the IP address and port you wish the server to bind to.

To run the Kademlia node as a client and perform operations:

```bash
cargo run -- client <TARGET_IP:PORT> <COMMAND>
```
**\<TARGET_IP:PORT\>**: Specify the target server's IP address and port.

**\<COMMAND\>**: The command to execute, such as ping.

### Example

Run a server:

```bash
cargo run -- server 127.0.0.1:50051
```
In another terminal, run a client to ping the server:

```bash
cargo run -- client 127.0.0.1:50051 ping
```
