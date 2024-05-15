use clap::{Command, Arg};
use tokio::runtime::Runtime; // Ensure Tokio is in your dependencies

mod kademlia {
    tonic::include_proto!("kademlia");
}

mod node;
mod client;
mod config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Kademlia Network")
        .version("1.0")
        .author("Eduardo Correia <eduardo.mmd.correia@gmail.com>")
        .about("Runs an S/Kademlia node as either a server or a client")
        .subcommand(Command::new("server")
            .about("Runs in server mode")
            .arg(Arg::new("addr")
                .help("The address to bind on")
                .required(true)
                .index(1))
            .arg(Arg::new("bootstrap")
                .help("Optional address of the bootstrap node")
                .long("bootstrap") // Named argument
                .value_parser(clap::value_parser!(String)) // Correct method for Clap v4 to accept a value
                .required(false)))
        .subcommand(Command::new("client")
            .about("Runs in client mode")
            .arg(Arg::new("addr")
                .help("The target address to connect to")
                .required(true)
                .index(1))
            .arg(Arg::new("command")
                .help("The command to execute on the client")
                .required(true)
                .index(2)))
        .get_matches();

    // Create a new Tokio runtime
    let rt = Runtime::new()?;

    match matches.subcommand() {
        Some(("server", server_matches)) => {
            let addr = server_matches.get_one::<String>("addr").unwrap(); // Extracts the server address
            let bootstrap_addr = server_matches.get_one::<String>("bootstrap"); // Extracts the optional bootstrap address
            rt.block_on(node::run_server(addr, bootstrap_addr.cloned()))?; // Use block_on to run the async function, passing the optional bootstrap address
        },
        Some(("client", client_matches)) => {
            let addr = client_matches.get_one::<String>("addr").unwrap(); // Extracts the client address
            let command = client_matches.get_one::<String>("command").unwrap(); // Extracts the command
            rt.block_on(client::run_client(addr, command))?; // Runs the client logic
        },
        _ => unreachable!(), // Ensures that the command falls into known subcommands
    }

    Ok(())
}
