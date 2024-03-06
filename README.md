
docker-compose up

export DATABASE_URL="postgres://testuser:testpassword@localhost/testdb"

<!-- cargo sqlx migrate run -->

<!-- cargo sqlx prepare -->

cargo run --release

- In another terminal upload some data

    ```
    curl -XPOST http://localhost:8080/ingest/ --header 'content-type: application/json' -d '{"text": "green iphone 18gb"}'

    curl -XPOST http://localhost:8080/ingest/ --header 'content-type: application/json' -d '{"text": "red iphone 18gb"}'

    curl -XPOST http://localhost:8080/ingest/ --header 'content-type: application/json' -d '{"text": "green iphone 12gb"}'

    curl -XPOST http://localhost:8080/ingest/ --header 'content-type: application/json' -d '{"text": "red iphone 12gb"}'
    ```

- Consolidate the results or wait a few seconds for the service to catch up itself

    ```
    curl -XPOST http://localhost:8080/consolidate
    ```

- Navigate to <http://localhost:8080> on your browser and type `green` or `red` etc.

### Adding db migrations

- Install `sqlx-cli`
  `cargo install sqlx-cli`

- Generate the migrations. Note `-r`, this is to generate `up` and `down` of your migrations.

  ```
  sqlx migrate add -r <name-of-your-migration>
  ```
