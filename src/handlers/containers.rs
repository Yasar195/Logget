use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, Sse},
    Json,
};
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::{sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    models::container::ContainerSummary,
    services::docker_service::{get_logs, list_containers_raw},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub follow: Option<bool>,
    pub tail: Option<String>,
}

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

pub async fn list_containers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ContainerSummary>>, StatusCode> {
    let containers = list_containers_raw(&state.docker).await?;
    Ok(Json(containers))
}

pub async fn search_containers(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<ContainerSummary>>, StatusCode> {
    let term = params.q.as_deref().unwrap_or("").trim().to_ascii_lowercase();
    let containers = list_containers_raw(&state.docker).await?;

    let filtered = if term.is_empty() {
        containers
    } else {
        containers
            .into_iter()
            .filter(|container| {
                let names = container
                    .names
                    .iter()
                    .map(|name| name.to_ascii_lowercase())
                    .collect::<Vec<_>>();
                let image = container.image.to_ascii_lowercase();
                let id = container.id.to_ascii_lowercase();

                names.iter().any(|name| name.contains(&term))
                    || image.contains(&term)
                    || id.contains(&term)
            })
            .collect()
    };

    Ok(Json(filtered))
}

pub async fn container_events(
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let docker = state.docker.clone();
    let (tx, rx) = mpsc::channel::<Result<axum::response::sse::Event, std::convert::Infallible>>(32);

    tokio::spawn(async move {
        let mut filters = HashMap::new();
        filters.insert("type".to_string(), vec!["container".to_string()]);

        let options = bollard::query_parameters::EventsOptionsBuilder::default()
            .filters(&filters)
            .build();

        let mut stream = docker.events(Some(options));

        while let Some(item) = stream.next().await {
            let event = match item {
                Ok(event) => event,
                Err(err) => {
                    let payload = serde_json::json!({
                        "type": "error",
                        "message": err.to_string(),
                    });
                    let _ = tx.send(Ok(axum::response::sse::Event::default().data(payload.to_string()))).await;
                    continue;
                }
            };

            if event.typ != Some(bollard::models::EventMessageTypeEnum::CONTAINER) {
                continue;
            }

            let action = event.action.clone().unwrap_or_default();
            let id = event
                .actor
                .as_ref()
                .and_then(|actor| actor.id.clone())
                .unwrap_or_default();
            let attributes = event
                .actor
                .as_ref()
                .and_then(|actor| actor.attributes.as_ref())
                .cloned()
                .unwrap_or_default();
            let name = attributes.get("name").cloned().unwrap_or_else(|| id.clone());

            let payload = serde_json::json!({
                "type": "container",
                "action": action,
                "id": id,
                "name": name,
                "attributes": attributes,
            });

            if tx.send(Ok(axum::response::sse::Event::default().data(payload.to_string()))).await.is_err() {
                break;
            }
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)).into_response())
}

pub async fn container_logs(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Response, StatusCode> {
    let follow = query.follow.unwrap_or(false);
    let tail = query.tail.unwrap_or_else(|| "100".to_string());

    if !follow {
        let logs = get_logs(&state.docker, &name, false, &tail).await?;
        return Ok(Json(serde_json::json!({
            "container": name,
            "logs": logs
        }))
        .into_response());
    }

    let docker = state.docker.clone();
    let (tx, rx) = mpsc::channel::<Result<axum::response::sse::Event, std::convert::Infallible>>(32);

    tokio::spawn(async move {
        let mut stream = docker.logs(
            &name,
            Some(
                bollard::query_parameters::LogsOptionsBuilder::default()
                    .follow(true)
                    .stdout(true)
                    .stderr(true)
                    .tail(&tail)
                    .build(),
            ),
        );

        while let Some(item) = stream.next().await {
            let event = match item {
                Ok(output) => axum::response::sse::Event::default().data(output.to_string()),
                Err(err) => axum::response::sse::Event::default().data(err.to_string()),
            };

            if tx.send(Ok(event)).await.is_err() {
                break;
            }
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)).into_response())
}
