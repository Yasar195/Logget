use axum::http::StatusCode;
use bollard::{
    query_parameters::{ListContainersOptionsBuilder, LogsOptionsBuilder},
    Docker,
};
use futures_util::StreamExt;

use crate::models::container::ContainerSummary;

pub async fn list_containers_raw(docker: &Docker) -> Result<Vec<ContainerSummary>, StatusCode> {
    let containers = docker
        .list_containers(Some(ListContainersOptionsBuilder::default().all(false).build()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(containers
        .into_iter()
        .map(|container| ContainerSummary {
            id: container.id.unwrap_or_default(),
            names: container
                .names
                .unwrap_or_default()
                .into_iter()
                .map(|name| name.trim_start_matches('/').to_string())
                .collect(),
            image: container.image.unwrap_or_default(),
            status: container.status.unwrap_or_default(),
            state: container
                .state
                .map(|value| format!("{:?}", value))
                .unwrap_or_default(),
        })
        .collect())
}

pub async fn get_logs(
    docker: &Docker,
    name: &str,
    follow: bool,
    tail: &str,
) -> Result<Vec<String>, StatusCode> {
    let mut stream = docker.logs(
        name,
        Some(
            LogsOptionsBuilder::default()
                .follow(follow)
                .stdout(true)
                .stderr(true)
                .tail(tail)
                .build(),
        ),
    );

    let mut collected = Vec::new();
    while let Some(item) = stream.next().await {
        let output = item.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        collected.push(output.to_string());
    }

    Ok(collected)
}
