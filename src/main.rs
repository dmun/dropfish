use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    File {
        #[command(subcommand)]
        command: FileCommand,
    },
}

#[derive(Subcommand, Debug)]
enum FileCommand {
    Put { source: String, destination: String },
    Get { path: String },
}

enum Backend {}

fn handle_file(command: FileCommand) -> Result<()> {
    match command {
        FileCommand::Put {
            source,
            destination,
        } => Ok(()),
        FileCommand::Get { path } => Ok(()),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::File { command } => handle_file(command),
    }
}
