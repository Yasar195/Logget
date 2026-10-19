use bollard::query_parameters::{ListContainersOptionsBuilder, LogsOptionsBuilder};
use bollard::Docker;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::connect_with_local_defaults()?;

    let containers = docker
        .list_containers(Some(ListContainersOptionsBuilder::default().all(false).build()))
        .await?;

    if let Some(container) = containers.first() {
        let name = container
            .names
            .as_ref()
            .and_then(|n| n.first())
            .map(|n| n.trim_start_matches('/'))
            .unwrap_or("unknown-container");

        println!("Following logs for: {}", name);

        let mut stream = docker.logs(
            name,
            Some(
                LogsOptionsBuilder::default()
                    .follow(true)
                    .stdout(true)
                    .stderr(true)
                    .tail("all")
                    .build(),
            ),
        );

        while let Some(log) = stream.next().await {
            let line = log?;
            println!("{}", line);
        }
    }

    Ok(())
}