# S/Kademlia DHT Node in Rust
This component implements a secure and robust version of a Kademlia Distributed Hash Table (DHT) node using Rust and gRPC. Kademlia is a decentralized peer-to-peer network protocol that enables nodes in the network to store and retrieve data in a distributed manner. This implementation focuses on the core functionalities of the Kademlia protocol, including node discovery, data storage, and retrieval, with an emphasis on security and reliability.

## Features
- Message Signing and Verification: Ensures the authenticity and integrity of messages exchanged between nodes.
- Reputation System: Maintains a reputation score for each node to ensure trustworthiness and remove unreliable nodes from the routing table.
- Node IP Diversification: Limits the number of nodes from the same IP address to prevent network centralization.
- Proof of Work for Node ID Generation: Implements a proof-of-work mechanism to generate node IDs, adding a computational cost to node creation.
- Secure PING: Probes a node to check its online status with message signing for verification.
- Secure STORE: Instructs a node to store a key-value pair with verification of the requester's identity.
- Secure FIND_NODE: Returns information about the k closest nodes to a given target node ID with requester validation.
- Secure FIND_VALUE: Similar to FIND_NODE, but returns the stored value for a given key if available, with requester validation.

## Flowchart Representation

```plaintext
[Start Node]
    ↓
[Initialize Node]
    |→ [Generate Node ID]
    |   |→ [Proof of Work]
    |→ [Set Up Routing Table]
    |→ [Fetch Routing Table from Bootstrap (optional)]
    ↓
[Run Server]
    ↓
[Listen for Requests]
    |→ [Handle `ping`]
    |→ [Handle `store`]
    |→ [Handle `find_node`]
    |   |→ [Update Routing Table with Requester Info (if new)]
    |   |→ [Return Closest Nodes]
    |→ [Handle `find_value`]
    ↓
[Periodic Routing Table Refresh]
    |→ [Select Random Node]
    |→ [Fetch and Merge Routing Table]
    ↓
[Maintain Node Connections]
    |→ [Validate Nodes]
    |→ [Remove Non-responsive Nodes]
    |→ [Adjust Node Reputation]
    |→ [Enforce IP Diversification]
    ↓
[Continue Listening for Requests]
```

## Security Features

### Message Signing and Verification
All messages sent between nodes are signed using Ed25519 cryptography, ensuring that the messages are not tampered with in transit and that they originate from a verified node.

### Reputation System
Each node maintains a reputation score for other nodes. The reputation score is adjusted based on the node's responsiveness and behavior. Nodes with a reputation below a predefined threshold are permanently banned and removed from the routing table, ensuring that only trustworthy nodes participate in the network.

### Node IP Diversification
To prevent centralization and potential Sybil attacks, the routing table enforces a limit on the number of nodes that can be added from the same IP address.

### Proof of Work for Node ID Generation
Node IDs are generated using a proof-of-work mechanism. This involves performing a computationally expensive task to generate an ID that meets certain criteria (e.g., a specific number of leading zeros). This makes it costly for an attacker to generate a large number of node IDs quickly, thus mitigating Sybil attacks.

### Secure PING, STORE, FIND_NODE, and FIND_VALUE
These core operations include verification steps to ensure that requests are coming from legitimate and verified nodes. This adds an extra layer of security to the network operations.

## Security Considerations

### Eclipse Attacks
Eclipse attacks involve isolating a target node by surrounding it with malicious nodes controlled by an attacker. This implementation mitigates eclipse attacks by:

- Enforcing IP diversification to prevent a single IP from dominating the routing table.
- Maintaining a reputation system to identify and remove unreliable or malicious nodes.

### Sybil Attacks
Sybil attacks involve an attacker creating multiple fake identities to control a large portion of the network. This implementation counters Sybil attacks by:

- Limiting the number of nodes from the same IP address.
- Using a reputation system to ensure that only trustworthy nodes remain in the network.
- Implementing proof-of-work for node ID generation to add a computational cost to node creation.

### Replay Attacks
Replay attacks involve an attacker intercepting and retransmitting a valid data transmission. This implementation prevents replay attacks by:

- Using timestamps in messages to ensure they are recent and not reused.
- Implementing nonce values for each message to ensure uniqueness.

### Man-in-the-Middle Attacks
Man-in-the-middle attacks involve an attacker intercepting communication between two nodes. This implementation mitigates such attacks by:

- Signing all messages with Ed25519 cryptography to ensure message integrity and authenticity.
- Verifying signatures on all incoming messages to ensure they originate from a legitimate source.

## Prerequisites
Before you begin, ensure you have met the following requirements:

- Rust and Cargo installed on your machine.
- gRPC and Protocol Buffers setup for Rust. The tonic crate is used for the gRPC implementation.

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

To run a Kademlia node as a server, with a bootstrap node:

```bash
cargo run -- server <IP:PORT> --bootstrap <IP:PORT>
```

To run the Kademlia node as a client and perform operations:

```bash
cargo run -- client <TARGET_IP:PORT> <COMMAND>
```