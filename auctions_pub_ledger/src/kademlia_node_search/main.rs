use std::option;

use clap::{Arg, Command};
use tokio::runtime::Runtime; // Ensure Tokio is in your dependencies

mod kademlia {
    tonic::include_proto!("kademlia");
}

use std::net::SocketAddr;
mod client;
mod config;
mod node;
use crate::node::Node;
use std::sync::Arc;
use tokio::sync::Mutex;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Kademlia Network")
        .version("1.0")
        .author("Eduardo Correia <eduardo.mmd.correia@gmail.com>")
        .about("Runs an S/Kademlia node as either a server or a client")
        .subcommand(
            Command::new("server")
                .about("Runs in server mode")
                .arg(
                    Arg::new("addr")
                        .help("The address to bind on")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("bootstrap")
                        .help("Optional address of the bootstrap node")
                        .value_parser(clap::value_parser!(String)) // Correct method for Clap v4 to accept a value
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("client")
                .about("Runs in client mode")
                .arg(
                    Arg::new("addr")
                        .help("The target address to connect to")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("command")
                        .help("The command to execute on the client")
                        .required(true)
                        .index(2),
                ),
        )
        .get_matches();

    let rt = Runtime::new()?;
    match matches.subcommand() {
        Some(("server", server_matches)) => {
            let addr = server_matches.get_one::<String>("addr").unwrap();
            let bootstrap_addr = server_matches.get_one::<String>("bootstrap");
            rt.block_on(async {
                let addr = addr.parse::<SocketAddr>().unwrap();
                let node = node::Node::new(addr.clone(), bootstrap_addr.cloned()).await;
                println!("{:?}", node);

                node::run_server(addr.clone(), node.unwrap()).await;
            });
            // Use block_on to run the async function, passing the optional bootstrap address
        }
        Some(("client", client_matches)) => {
            let addr = client_matches.get_one::<String>("addr").unwrap(); // Extracts the client address
            let command = client_matches.get_one::<String>("command").unwrap(); // Extracts the command
            rt.block_on(client::run_client(addr, command))?; // Runs the client logic
        }
        _ => unreachable!(), // Ensures that the command falls into known subcommands
    }

    Ok(())
}
