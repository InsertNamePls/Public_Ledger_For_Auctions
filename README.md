# Public_Ledger_For_Auctions

### S-Kademlia DHS
#### Algorithm

### Public ledger

task!!!!!
test encryption and signiing messages with public and pivate key
sign(message, sk)=signature
verify(message, signature, pk)=(true/false)
#### Algorithm
using hashsha 256
1 -> New transactions are broadcast to all nodes.
2 -> Each node collects new transactions into a block.
3 -> Each node works on finding a difficult proof-of-work for its block.
4 -> When a node finds a proof-of-work, it broadcasts the block to all nodes.
5 -> Nodes accept the block only if all transactions in it are valid and not already spent.
6 -> Nodes express their acceptance of the block by working on creating the next block in the
chain, using the hash of the accepted block as the previous hash.




1 -> generate genesis block
2 -> hash the block ( with 2 0 at the begining) sha-256
4 -> calculate proof of work of that block (nonce)
3 -> send the block to to every node


### Auction House dedicated to sell items
#### Algorithm

(The auction activity is made through cli)
1 -> User authenticate using an User name and a ssh key pair.
    Without the keys is impossible to verify the authenticity and integrity of the user so if the keys get lost all the items accuired are also lost.
2 -> user can:
    * list items in auction (even those that already expired)
    * Create a bid on an active auction if the user has enought money on the wallet
    * Sell an item.
    * check current balance
3 -> auctions take 10 minute to expire.
4 -> every time a user executes a bid under 30s the auction extends the end    time +30s.
5 -> every operation in the auction involve a creation of a new block?(I Assume yes!)

#### commands
```bash
$ acution --login --user <username> --priv_key <private_key_path> --pub_key <pub_key_path>
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


####

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
