# Logget

<img src="./logget.png" alt="Logget logo" width="200">

Logget is a lightweight Rust application for watching Docker container activity and streamings live logs from running containers.

## What it does

- Lists running Docker containers
- Chooses the first available container
- Follows that container's stdout/stderr logs in real time
- Prints log output as it arrives

## Tech stack

- Rust
- Tokio
- Bollard
- Docker API

## Requirements

- Docker installed and running on the host
- Linux/macOS environment with Docker socket accesss
- Rust toolchain if you want to build locally
- Target Linux machine must allow the binary to access `/var/run/docker.sock`

## Local run

```bash
cd logget
cargo run
```

This connects to the local Docker daemon and starts following logs for the first available running container.

## Download latest Linux binary

### Direct curl download

```bash
curl -fsSL https://github.com/Yasar195/Logget/releases/latest/download/logget-linux-amd64.tar.gz -o logget-linux-amd64.tar.gz
tar -xzf logget-linux-amd64.tar.gz
sudo install -m 0755 logget-linux-amd64 /usr/local/bin/logget
```

### Install script

```bash
curl -fsSL https://raw.githubusercontent.com/Yasar195/Logget/master/install.sh | sudo bash -s -- Yasar195/Logget
```

Or with an environment variable:

```bash
curl -fsSL https://raw.githubusercontent.com/Yasar195/Logget/master/install.sh | sudo env LOGGET_REPO=Yasar195/Logget bash
```

## Docker run

From the project root:

```bash
docker build -t logget .
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock logget
```

## Notes

This project is intentionally simple and intended for local monitoring of Docker containers. If you want to monitor a specific container instead of the first one, the code can be extended to accept a container name or ID as an argument.

## Example output

```text
Following logs for: my-container
2026-10-19T12:00:00Z app started
2026-10-19T12:00:01Z serving requests
```

## Future improvements

- Support selecting a container by name or ID
- Show container list with names and statuses
- Filter logs by timestamp or severity
- Add a web UI or dashboard
- Support multiple container streams concurrently

## Release workflow

This repo publishes a Linux x86_64 release binary using GitHub Actions. Every git tag matching `v*` triggers a build and uploads the resulting release assets.

Example:

```bash
git tag v1.0.0
git push origin v1.0.0
```
