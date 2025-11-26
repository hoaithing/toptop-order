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
                        payment TEXT,
                        recipient_address TEXT,
                        item_list TEXT,
                        fulfillment_type TEXT,
                        warehouse_id TEXT,
                        buyer_message TEXT
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
            let payment_json = serde_json::to_string(&order.payment).unwrap_or_default();
            let recipient_address_json =
                serde_json::to_string(&order.recipient_address).unwrap_or_default();
            let item_list_json = serde_json::to_string(&order.item_list).unwrap_or_default();

            self.conn
                .call(move |conn| {
                    conn.execute(
                        "INSERT OR REPLACE INTO orders (
                            id, status, create_time, update_time, payment, recipient_address, 
                            item_list, fulfillment_type, warehouse_id, buyer_message
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        params![
                            order.id,
                            order.status,
                            order.create_time,
                            order.update_time,
                            payment_json,
                            recipient_address_json,
                            item_list_json,
                            order.fulfillment_type,
                            order.warehouse_id,
                            order.buyer_message,
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
                let mut stmt = conn.prepare("SELECT id, status, create_time, update_time, payment, recipient_address, item_list, fulfillment_type, warehouse_id, buyer_message FROM orders")?;
                let order_iter = stmt.query_map([], |row| {
                    let payment_json: String = row.get(4)?;
                    let recipient_address_json: String = row.get(5)?;
                    let item_list_json: String = row.get(6)?;

                    Ok(Order {
                        id: row.get(0)?,
                        status: row.get(1)?,
                        create_time: row.get(2)?,
                        update_time: row.get(3)?,
                        payment: serde_json::from_str(&payment_json).unwrap_or_default(),
                        recipient_address: serde_json::from_str(&recipient_address_json)
                            .unwrap_or_default(),
                        item_list: serde_json::from_str(&item_list_json).unwrap_or_default(),
                        fulfillment_type: row.get(7)?,
                        warehouse_id: row.get(8)?,
                        buyer_message: row.get(9)?,
                        // The following fields are not stored in the database
                        buyer_email: None,
                        cancel_order_sla_time: None,
                        cancel_reason: None,
                        cancel_time: None,
                        cancellation_initiator: None,
                        collection_due_time: None,
                        commerce_platform: None,
                        delivery_option_id: None,
                        delivery_option_name: None,
                        delivery_type: None,
                        has_updated_recipient_address: None,
                        is_cod: None,
                        is_on_hold_order: None,
                        is_replacement_order: None,
                        is_sample_order: None,
                        order_type: None,
                        packages: Vec::new(),
                        paid_time: None,
                        payment_method_name: None,
                        rts_sla_time: None,
                        rts_time: None,
                        shipping_due_time: None,
        
                        shipping_provider: None,
                        shipping_provider_id: None,
                        shipping_type: None,
                        tracking_number: None,
                        tts_sla_time: None,
                        user_id: None,
                        collection_time: None,
                        delivery_time: None,
                    })
                })?;

                let mut orders = Vec::new();
                for order in order_iter {
                    orders.push(order?);
                }
                Ok(orders)
            })
            .await
    }
}
