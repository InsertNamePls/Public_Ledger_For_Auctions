use clap::{App, Arg, SubCommand};
use std::error::Error;

mod kademlia {
    tonic::include_proto!("kademlia");
}

mod node;
mod client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Kademlia Network")
        .version("1.0")
        .author("Author Name <author@example.com>")
        .about("Runs a Kademlia node as either a server or a client")
        .subcommand(SubCommand::with_name("server")
            .about("Runs in server mode")
            .arg(Arg::with_name("addr")
                .help("The address to bind on")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("client")
            .about("Runs in client mode")
            .arg(Arg::with_name("addr")
                .help("The target address to connect to")
                .required(true)
                .index(1))
            .arg(Arg::with_name("command")
                .help("The command to execute on the client")
                .required(true)
                .index(2)))
        .get_matches();

        match matches.subcommand() {
            Some(("server", server_matches)) => {
                let addr = server_matches.value_of("addr").unwrap();
                node::run_server(addr).await?;
            },
            Some(("client", client_matches)) => {
                let addr = client_matches.value_of("addr").unwrap();
                let command = client_matches.value_of("command").unwrap();
                client::run_client(addr, command).await?;
            },
            _ => unreachable!(), // Since clap ensures that we don't get here due to the defined subcommands
        }
        

    Ok(())
}
