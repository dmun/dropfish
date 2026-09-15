use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use serde::Deserialize;

use crate::config::Config;
use crate::server::serve;

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
    Serve {},
}

#[derive(Subcommand, Debug)]
enum FileCommand {
    Upload { source: String, destination: String },
    Download { path: String },
}

async fn handle_file(config: Config, command: FileCommand) -> Result<()> {
    let endpoint = format!("http://{}/file", config.address());
    let client = reqwest::Client::new();

    match command {
        FileCommand::Upload {
            source,
            destination,
        } => {
            let contents = tokio::fs::read(source).await?;
            client
                .put(endpoint)
                .query(&[("destination", destination)])
                .body(contents)
                .send()
                .await?
                .error_for_status()?;
        }
        FileCommand::Download { path: destination } => {
            let result = client
                .get(endpoint)
                .query(&[("destination", destination)])
                .send()
                .await?
                .error_for_status()?;

            dbg!(result.text().await?);
        }
    };

    Ok(())
}

pub async fn run() -> Result<()> {
    let args = Args::parse();
    let config = Config::default();

    match args.command {
        Command::File { command } => handle_file(config, command).await,
        Command::Serve {} => serve(config).await,
    }?;

    Ok(())
}
