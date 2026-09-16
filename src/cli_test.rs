use std::time::Duration;

use anyhow::Result;
use tempfile::TempDir;
use tokio::{net::TcpListener, task::JoinHandle, time::timeout};

use super::{download, upload};
use crate::{config::Config, server};

struct TestServer {
    endpoint: String,
    storage: TempDir,
    task: JoinHandle<std::io::Result<()>>,
}

impl TestServer {
    async fn start() -> Result<Self> {
        let storage = tempfile::tempdir()?;
        let mut config = Config::default();
        config.location = storage.path().to_path_buf();

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}/file", listener.local_addr()?);
        let task = tokio::spawn(async move { axum::serve(listener, server::app(config)).await });

        Ok(Self {
            endpoint,
            storage,
            task,
        })
    }

    async fn stop(mut self) {
        self.task.abort();
        match (&mut self.task).await {
            Ok(result) => result.expect("test server failed"),
            Err(error) if error.is_cancelled() => {}
            Err(error) => panic!("test server task failed: {error}"),
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[tokio::test]
async fn upload_then_download() -> Result<()> {
    let server = TestServer::start().await?;
    let local = tempfile::tempdir()?;
    let source = local.path().join("original.bin");
    let contents = b"hello dropfish\x00\xff";
    std::fs::write(&source, contents)?;

    timeout(Duration::from_secs(5), async {
        upload(&server.endpoint, &source, "docs/file.bin").await?;

        assert_eq!(
            std::fs::read(server.storage.path().join("docs/file.bin"))?,
            contents
        );
        assert_eq!(download(&server.endpoint, "docs/file.bin").await?, contents);

        anyhow::Ok(())
    })
    .await??;

    server.stop().await;
    Ok(())
}
