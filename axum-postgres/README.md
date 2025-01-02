# axum-postgres

This is an example of a RESTful API built using the Axum framework (https://github.com/tokio-rs/axum) with PostgreSQL as the database.

## Setup

To set up the project, follow these steps:

1. Clone the repository:
    ```sh
    git clone <repository-url>
    cd axum-postgres
    ```

2. Build and run the development environment using Docker:
    ```sh
    docker compose up --build
    ```

## Endpoints

- `GET /api/v1/todos`: Retrieves a list of all todo items.
  ```sh
  curl -X GET http://localhost:3000/api/v1/todos
  ```

- `POST /api/v1/todos`: Adds a new todo item to the collection.
  ```sh
  curl -X POST http://localhost:3000/api/v1/todos \
       -H "Content-Type: application/json" \
       -d '{"text":"Title"}'
  ```

- `POST /api/v1/todos/{todo_id}`: Updates the todo item in the collection.
  ```sh
  curl -X POST http://localhost:3000/api/v1/todos/{todo_id} \
       -H "Content-Type: application/json" \
       -d '{"text":"New title"}'
  ```

- `DELETE /api/v1/todos/{todo_id}`: Deletes the todo item from the collection.
  ```sh
  curl -X DELETE http://localhost:3000/api/v1/todos/{todo_id}
  ```
