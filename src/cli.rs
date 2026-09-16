use std::path::Path;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use crate::config::Config;
use crate::server::serve;

#[cfg(test)]
#[path = "cli_test.rs"]
mod tests;

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

async fn upload(endpoint: &str, source: &Path, destination: &str) -> Result<()> {
    let contents = tokio::fs::read(source).await?;
    reqwest::Client::new()
        .put(endpoint)
        .query(&[("destination", destination)])
        .body(contents)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

async fn download(endpoint: &str, destination: &str) -> Result<Vec<u8>> {
    let response = reqwest::Client::new()
        .get(endpoint)
        .query(&[("destination", destination)])
        .send()
        .await?
        .error_for_status()?;
    Ok(response.bytes().await?.to_vec())
}

async fn handle_file(config: Config, command: FileCommand) -> Result<()> {
    let endpoint = format!("http://{}/file", config.address());

    match command {
        FileCommand::Upload {
            source,
            destination,
        } => {
            upload(&endpoint, Path::new(&source), &destination).await?;
        }
        FileCommand::Download { path: destination } => {
            let contents = download(&endpoint, &destination).await?;
            dbg!(String::from_utf8_lossy(&contents));
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
