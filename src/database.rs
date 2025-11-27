use crate::order::Order;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(path: &str) -> Result<Self, sqlx::Error> {
        // Ensure the database file can be created
        let database_url = format!("sqlite:{}?mode=rwc", path);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                create_time INTEGER NOT NULL,
                update_time INTEGER NOT NULL,
                data TEXT NOT NULL,
                synced_at INTEGER NOT NULL
            )"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert or update orders in the database
    pub async fn upsert_orders(&self, orders: &[Order]) -> Result<(), sqlx::Error> {
        for order in orders {
            let order_json = serde_json::to_string(&order)
                .unwrap_or_default();
            let synced_at = chrono::Utc::now().timestamp();

            sqlx::query(
                "INSERT OR REPLACE INTO orders (
                    id, status, create_time, update_time, data, synced_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )
            .bind(&order.id)
            .bind(&order.status)
            .bind(order.create_time)
            .bind(order.update_time)
            .bind(&order_json)
            .bind(synced_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get all orders from the database
    pub async fn get_orders(&self) -> Result<Vec<Order>, sqlx::Error> {
        let rows = sqlx::query("SELECT data FROM orders ORDER BY create_time DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut orders = Vec::new();
        for row in rows {
            let data_json: String = row.try_get("data")?;
            if let Ok(order) = serde_json::from_str::<Order>(&data_json) {
                orders.push(order);
            }
        }

        Ok(orders)
    }

    /// Get a single order by ID
    pub async fn get_order_by_id(&self, order_id: &str) -> Result<Option<Order>, sqlx::Error> {
        let row = sqlx::query("SELECT data FROM orders WHERE id = ?1")
            .bind(order_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let data_json: String = row.try_get("data")?;
            if let Ok(order) = serde_json::from_str::<Order>(&data_json) {
                return Ok(Some(order));
            }
        }

        Ok(None)
    }

    /// Get the total count of orders
    pub async fn get_orders_count(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM orders")
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.try_get("count")?;
        Ok(count)
    }

    /// Get orders with pagination
    pub async fn get_orders_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Order>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT data FROM orders ORDER BY create_time DESC LIMIT ?1 OFFSET ?2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::new();
        for row in rows {
            let data_json: String = row.try_get("data")?;
            if let Ok(order) = serde_json::from_str::<Order>(&data_json) {
                orders.push(order);
            }
        }

        Ok(orders)
    }

    /// Get orders by status
    pub async fn get_orders_by_status(&self, status: &str) -> Result<Vec<Order>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT data FROM orders WHERE status = ?1 ORDER BY create_time DESC"
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::new();
        for row in rows {
            let data_json: String = row.try_get("data")?;
            if let Ok(order) = serde_json::from_str::<Order>(&data_json) {
                orders.push(order);
            }
        }

        Ok(orders)
    }

    /// Delete an order by ID
    pub async fn delete_order(&self, order_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM orders WHERE id = ?1")
            .bind(order_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
