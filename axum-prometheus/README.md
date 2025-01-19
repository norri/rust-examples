# axum-prometheus

This is an example of a RESTful API built using the Axum framework (https://github.com/tokio-rs/axum) with Prometheus metrics.

## Setup

To set up the project, follow these steps:

1. Clone the repository:
    ```sh
    git clone <repository-url>
    cd axum-prometheus
    ```

2. Build and run the development environment using Docker:
    ```sh
    docker compose up --build
    ```

## Endpoints

- `GET /fast`: Returns OK immediately
  ```sh
  curl -X GET http://localhost:8080/fast
  ```

- `GET /slow`: Returns OK after 1 second
  ```sh
  curl -X GET http://localhost:8080/slow
  ```

- `GET /error`: Returns HTTP 500 error
  ```sh
  curl -X GET http://localhost:8080/error
  ```

## Metrics

Access Grafana Dashboard in http://localhost:3000
