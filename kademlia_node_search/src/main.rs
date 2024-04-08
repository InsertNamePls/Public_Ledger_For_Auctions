use clap::{Command, Arg};
use tokio::runtime::Runtime; // Make sure you have 'tokio' in your dependencies

mod kademlia {
    tonic::include_proto!("kademlia");
}

mod node;
mod client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Kademlia Network")
        .version("1.0")
        .author("Author Name <author@example.com>")
        .about("Runs a Kademlia node as either a server or a client")
        .subcommand(Command::new("server")
            .about("Runs in server mode")
            .arg(Arg::new("addr")
                .help("The address to bind on")
                .required(true)
                .index(1)))
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
            let addr = server_matches.get_one::<String>("addr").unwrap(); // Adjusted for clap v4
            rt.block_on(node::run_server(addr))?; // Use block_on to run the async function
        },
        Some(("client", client_matches)) => {
            let addr = client_matches.get_one::<String>("addr").unwrap(); // Adjusted for clap v4
            let command = client_matches.get_one::<String>("command").unwrap(); // Adjusted for clap v4
            rt.block_on(client::run_client(addr, command))?; // Use block_on to run the async function
        },
        _ => unreachable!(), // Since clap ensures that we don't get here due to the defined subcommands
    }

    Ok(())
}