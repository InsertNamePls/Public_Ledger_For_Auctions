# Public Ledger For Auctions

## S-Kademlia DHS
### Algorithm

### Public Ledger

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
