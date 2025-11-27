use crate::order::Order;
use rusqlite::params;
use tokio_rusqlite::Connection;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self, tokio_rusqlite::Error> {
        let conn = Connection::open(path).await?;
        Ok(Self { conn })
    }

    pub async fn init(&self) -> Result<(), tokio_rusqlite::Error> {
        self.conn
            .call(|conn| {
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS orders (
                        id TEXT PRIMARY KEY,
                        status TEXT NOT NULL,
                        create_time INTEGER NOT NULL,
                        update_time INTEGER NOT NULL,
                        data TEXT NOT NULL,
                        synced_at INTEGER NOT NULL
                    )",
                    [],
                )?;
                Ok(())
            })
            .await
    }

    pub async fn upsert_orders(&self, orders: &[Order]) -> Result<(), tokio_rusqlite::Error> {
        for order in orders {
            let order = order.clone();
            let order_json = serde_json::to_string(&order).unwrap_or_default();
            let synced_at = chrono::Utc::now().timestamp();

            self.conn
                .call(move |conn| {
                    conn.execute(
                        "INSERT OR REPLACE INTO orders (
                            id, status, create_time, update_time, data, synced_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            order.id,
                            order.status,
                            order.create_time,
                            order.update_time,
                            order_json,
                            synced_at,
                        ],
                    )?;
                    Ok(())
                })
                .await?;
        }
        Ok(())
    }

    pub async fn get_orders(&self) -> Result<Vec<Order>, tokio_rusqlite::Error> {
        self.conn
            .call(|conn| {
                let mut stmt = conn.prepare("SELECT data FROM orders ORDER BY create_time DESC")?;
                let order_iter = stmt.query_map([], |row| {
                    let data_json: String = row.get(0)?;
                    Ok(data_json)
                })?;

                let mut orders = Vec::new();
                for order_json in order_iter {
                    let json = order_json?;
                    if let Ok(order) = serde_json::from_str::<Order>(&json) {
                        orders.push(order);
                    }
                }
                Ok(orders)
            })
            .await
    }

    pub async fn get_order_by_id(&self, order_id: &str) -> Result<Option<Order>, tokio_rusqlite::Error> {
        let order_id = order_id.to_string();
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT data FROM orders WHERE id = ?1")?;
                let mut rows = stmt.query(params![order_id])?;

                if let Some(row) = rows.next()? {
                    let data_json: String = row.get(0)?;
                    if let Ok(order) = serde_json::from_str::<Order>(&data_json) {
                        return Ok(Some(order));
                    }
                }
                Ok(None)
            })
            .await
    }

    pub async fn get_orders_count(&self) -> Result<i64, tokio_rusqlite::Error> {
        self.conn
            .call(|conn| {
                let count: i64 = conn.query_row("SELECT COUNT(*) FROM orders", [], |row| row.get(0))?;
                Ok(count)
            })
            .await
    }
}
