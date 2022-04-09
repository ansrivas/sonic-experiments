use crate::errors::SonicErrors;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;
use sqlx::Row;
use sqlx::{self, Connection, PgPool};
use sqlx::{postgres::PgArguments, Arguments};
use sqlx::{FromRow, IntoArguments};

/// Run database migrations, this can also be switched to a separate app
pub async fn run_migrations(pool: &PgPool) -> Result<(), SonicErrors> {
    let _ = sqlx::migrate!().run(pool).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct Product {
    pub id: i64,
    pub details: String,
    pub object_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct RowCount {
    pub count: i64,
}

pub struct Postgres;

impl Postgres {
    /// Setup will create a new PgPool
    pub async fn setup(dsn: &str, max_conn: u32, db_schema: &str) -> Result<PgPool, SonicErrors> {
        let db_schema = db_schema.to_string();
        let pgpool = PgPoolOptions::new()
            .after_connect({
                move |conn| {
                    let create_schema = format!("CREATE SCHEMA IF NOT EXISTS {};", db_schema);
                    let set_schema_query = format!("SET search_path = '{}';", db_schema);
                    Box::pin(async move {
                        conn.execute(create_schema.as_ref()).await?;
                        conn.execute(set_schema_query.as_ref()).await?;
                        Ok(())
                    })
                }
            })
            .max_connections(max_conn)
            .connect(dsn)
            .await?;
        Ok(pgpool)
    }

    /// Ping the db and check if its alive
    /// Intended to be used in the health-check handler
    pub async fn ping(pool: &PgPool) -> Result<(), SonicErrors> {
        let mut conn = pool.acquire().await?;
        conn.ping().await?;
        drop(conn);
        Ok(())
    }

    /// Insert the product in the database
    pub async fn insert_product(pool: &PgPool, product: &Product) -> Result<Product, SonicErrors> {
        let product = sqlx::query_as::<_, Product>(
            r#"INSERT INTO product (details, object_id) VALUES($1,$2) RETURNING *"#,
        )
        .bind(&product.details)
        .bind(&product.object_id)
        .fetch_one(pool)
        .await?;
        Ok(product)
    }

    /// Query the products from the database
    pub async fn query_products(
        pool: &PgPool,
        object_ids: Vec<String>,
    ) -> Result<Vec<Product>, SonicErrors> {
        let ids: Vec<uuid::Uuid> = object_ids
            .iter()
            .map(|u| uuid::Uuid::parse_str(u).unwrap())
            .collect();
        let products = sqlx::query_as!(
            Product,
            r#"SELECT 
                *
            FROM product 
            WHERE object_id = ANY($1)"#,
            &ids[..]
        )
        .fetch_all(pool)
        .await?;
        Ok(products)
    }

    /// Get the table count
    pub async fn count(pool: &PgPool) -> Result<RowCount, SonicErrors> {
        // let table_name_split: Vec<&str> = table_name.split_ascii_whitespace().collect();
        // if table_name_split.len() > 1 {
        //     return Err(SonicErrors::Custom("Invalid table name found".to_string()));
        // }
        let count: RowCount =
            sqlx::query_as::<_, RowCount>(r#"SELECT count(*) from product as count"#)
                .fetch_one(pool)
                .await?;
        Ok(count)
    }

    /// Helper method for setting up test db
    /// If env-var CI_DOCKER is enabled i.e. the tests are
    /// running in CI, then use the host as `postgres-s` else
    /// `localhost`.
    pub async fn setup_test_db_pool(schema: &str) -> PgPool {
        let db_host = match std::env::var("CI_DOCKER") {
            Ok(_) => "postgres-s",
            Err(_) => "localhost",
        };
        let db_url = format!(
            "postgres://testuser:testpassword@{host}/testdb",
            host = db_host
        );
        Postgres::setup(&db_url, 4, schema)
            .await
            .expect("Failed to connect to DB")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils;

    #[tokio::test]
    async fn test_ping() {
        let schema = format!("sonic{}", utils::random_chars_without_prefix(5));
        let pool = Postgres::setup_test_db_pool(&schema).await;
        Postgres::ping(&pool).await.unwrap();
        assert!(true);
    }

    #[tokio::test]
    async fn test_insert_product() {
        let pool = Postgres::setup_test_db_pool("public").await;
        run_migrations(&pool).await.unwrap();

        let object_id = uuid::Uuid::new_v4();
        let product = Product {
            details: "Red Iphone".to_string(),
            object_id: object_id,
            ..Default::default()
        };
        Postgres::insert_product(&pool, &product).await.unwrap();
        let products = Postgres::query_products(&pool, vec![object_id.to_string()])
            .await
            .unwrap();
        assert!(products.len() == 1, "len is {}", products.len());
        assert!(true);
    }
}
