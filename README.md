# Public Ledger For Auctions

## S-Kademlia DHS
### Algorithm

### Public Ledger
# Introduction

This project embarks on the ambitious task of implementing a public, non-permissioned blockchain, distinct from the likes of Bitcoin and Ethereum. Its core objective is to establish a decentralized public ledger, meticulously designed to record auction transactions. Unlike conventional blockchain applications focused primarily on financial transactions, this initiative pivots towards creating a transparent and secure platform for auctioneering, blending the realms of blockchain technology with the dynamic world of auctions.

The development of this blockchain is structured into three pivotal components, each serving a distinct yet interconnected function within the ecosystem:

- **Distributed Ledger**: At the heart of this project lies the secure ledger, a modular entity that embraces flexibility by supporting both Proof of Work (PoW) and Delegated Proof of Stake (DPoS) as its cornerstone consensus mechanisms. This dual approach allows for:
  - Integration of **Proof of Work (PoW)**, emphasizing computational commitment and security.
  - Incorporation of **Proof of Stake (PoS)** mechanisms, leveraging stake-based voting and reputation systems to enhance network integrity.

- **Secure P2P Communication Layer**: Essential to the ledger's operation is a robust Peer-to-Peer (P2P) network layer. This layer is tasked with disseminating transaction data across the blockchain, ensuring that information is both secure and universally accessible. Key features include:
  - Implementation of **S/Kademlia** protocol, enhancing network efficiency and node discoverability.
  - Mechanisms to thwart **Sybil and Eclipse attacks**, safeguarding the network against malicious entities.
  - Integration of **trust mechanisms** within the PoS framework, fostering a reliable and cooperative network environment.

- **Auction System**: Central to this project is the auction mechanism, designed to facilitate interactions between sellers and buyers through a single attribute auction model, reminiscent of the English auction system. This component is characterized by:
  - Storing transactions within the blockchain using **public-key cryptography**, ensuring the authenticity and integrity of auction data.
  - A **publisher/subscriber model** built atop the Kademlia protocol to efficiently manage auction events, enhancing user experience and participation.

- **Fault Injection Mechanism**: To demonstrate the system's resilience and robustness, a fault injection mechanism is incorporated, allowing for the controlled shutdown of nodes. This feature is crucial for validating the system's durability and its capacity to withstand and recover from adverse conditions.

Together, these components synergize to form a comprehensive blockchain solution tailored for the auction industry, promising a revolution in how auctions are conducted, perceived, and trusted on a global scale.

**Task:**
- Test encryption and signing messages with public and private keys.
  - `sign(message, sk) = signature`
  - `verify(message, signature, pk) = (true/false)`

#### Algorithm
Using SHA-256:
1. New transactions are broadcast to all nodes.
2. Each node collects new transactions into a block.
3. Each node works on finding a difficult proof-of-work for its block.
4. When a node finds a proof-of-work, it broadcasts the block to all nodes.
5. Nodes accept the block only if all transactions in it are valid and not already spent.
6. Nodes express their acceptance of the block by working on creating the next block in the chain, using the hash of the accepted block as the previous hash.

Steps to initiate:
1. Generate the genesis block.
2. Hash the block (with two zeros at the beginning) using SHA-256.
3. Calculate the proof of work of that block (nonce).
4. Send the block to every node.

## Auction House Dedicated to Selling Items
### Algorithm

The auction activity is conducted through CLI (Command Line Interface).

1. Users authenticate using a username and an SSH key pair. Without the keys, it is impossible to verify the authenticity and integrity of the user, so if the keys are lost, all the acquired items are also lost.
2. Users can:
   - List items in auction (including those that have already expired).
   - Create a bid on an active auction if the user has enough money in the wallet.
   - Sell an item.
   - Check the current balance.
3. Auctions take 10 minutes to expire.
4. Every time a user executes a bid under 30 seconds, the auction extends the end time by +30 seconds.
5. Every operation in the auction involves the creation of a new block. (Assumed yes!)

### Commands

```bash
$ auction --login --user <username> --priv_key <private_key_path> --pub_key <pub_key_path>
```
```bash
$ auction --wallet
```
```bash
$ auction --item <item_id> --bid <value>
```
```bash
$ auction --sell-item <item_id> --bid <value>
```
Example output for auction --list-items:
```bash
$ auction --list-items
[{
    id: 0,
    name: "paint",
    price: 1000,
    start_date: 2024-01-01:43:16.55,
    end_date: 2024-01-01:43:18.55
    state: active
},
{
    id: 1,
    name: "desk",
    price: 300,
    start_date: 2024-01-01:43:16.55,
    end_date: 2024-07-01:43:18.55
    state: active
},
{
    id: 2,
    name: "chair",
    price: 80,
    start_date: 2024-01-01:43:16.55,
    end_date: 2024-01-07:43:19.55
    state: active
}]
```

### Auction Process

1. **Service Request**: The customer sends a service request to the responsible broker, facilitated by a set of peers. The broker responds with the current bid price (ask price).
   
2. **Price Offer**:
   - If no offer exceeds the current bid price (ask price), the offer is either dropped or stored for later consideration.
   - If a matching offer is found, it's forwarded to the peer with the highest buy price (or lowest sell price). The final price is determined as the mean between these matching offers.

#### Operations Include:
- Creation of an auction.
- Opening an auction.
- Bidding on items.
- Purchasing items.
1-> customer send a service request to the responsible broker, which is realized by a set of peers as described later on.
The broker replies with the current bid price (ask price).

2-> Based on this information, the customer
sends a price offer to the broker

2.1-> if there is no offer higher than the
current bid price (ask price). The price offer is dropped or
stored in a table for later use.

2.2-> If there is a match, the price offer is forwarded to the peer
that offered the highest buy price (lowest sell price). The
resulting price for the service is set to the mean price between
the matching price offers.


criação de um leilão
abertura de leilão
bid
compra do item



sudo apt-get update
sudo apt install build-essential
brew install protobuf


### Test Locally with docker
1. docker image build 
```bash docker build . --tag dledger2auction```

2. crate docker Instance
```bash docker run --name=test1 -dit dledger2auction && docker exec -it test1 bash ```

3. create instance test 1
```bash
cd home/auction_app
cargo build
cargo run

cd home/public_ledger
cargo build
cargo run --bin blockchain_operator -- init_blockchain 172.17.0.3
```
4. create instance test 2
```bash
cd home/public_ledger
cargo build
cargo run --bin blockchain_operator -- join_blockchain 172.17.0.2
```
5. the keypair is located in /home/public_key
#### stop docker instances
```bash
docker stop test1 test2 && docker rm test1 test2

remove recent image
docker image rm `docker images | grep dledger2auction | awk '{print $3}'`
```
