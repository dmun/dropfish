use std::path::Path;
use std::sync::mpsc::channel;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use notify::Event;
use notify::EventKind;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify::Watcher;
use notify::event::ModifyKind;

use crate::config;
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
    Sync {},
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

async fn sync_files(config: Config) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| {
            tx.send(result).expect("failed to send event");
        },
        notify::Config::default(),
    )?;

    watcher.watch(config.client_dir.as_path(), RecursiveMode::Recursive)?;

    for result in rx {
        match result {
            Ok(event) => handle_event(event).await?,
            Err(error) => println!("Error {}", error),
        }
    }

    Ok(())
}

async fn handle_event(event: Event) -> anyhow::Result<()> {
    match event.kind {
        EventKind::Modify(modify_kind) => handle_modify(&event, modify_kind).await,
        _ => Ok(()),
    }
}

async fn handle_modify(event: &Event, modify_kind: ModifyKind) -> anyhow::Result<()> {
    for x in &event.paths {
        // println!("{:?}: {:?}", modify_kind, x);
    }

    match modify_kind {
        ModifyKind::Data(content) => {
            let path = &event.paths[0];
            path.strip_prefix(config::Config::default().client_dir);
            match tokio::fs::read(path).await {
                Ok(contents) => {
                    // println!("{:?}", String::from_utf8(contents));
                    tokio::fs::write(path, contents).await?;
                }
                Err(error) => {
                    println!("{}", error);
                }
            };
        }
        _ => {}
    };

    Ok(())
}

pub async fn run() -> Result<()> {
    let args = Args::parse();
    let config = Config::default();

    match args.command {
        Command::File { command } => handle_file(config, command).await,
        Command::Serve {} => serve(config).await,
        Command::Sync {} => sync_files(config).await,
    }?;

    Ok(())
}
