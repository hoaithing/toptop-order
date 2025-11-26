use crate::error::AppError;
use crate::requests::TikTokShopApiClient;
use serde::{Deserialize, Serialize};

/// Order management client
pub struct OrderClient {
    api_client: TikTokShopApiClient,
}

impl OrderClient {
    pub fn new(app_key: String, app_secret: String) -> Self {
        Self {
            api_client: TikTokShopApiClient::new(app_key, app_secret),
        }
    }

    /// Get order list with filtering and pagination
    ///
    /// API endpoint: /api/orders/search
    /// Version: 202309
    pub async fn get_order_list(
        &self,
        access_token: &str,
        shop_cipher: Option<&str>,
        request: GetOrderListRequest,
    ) -> Result<GetOrderListResponse, AppError> {
        // Build request body
        let body = GetOrderListRequestBody::from(request);

        self.api_client
            .post("/api/orders/search", Some(access_token), shop_cipher, &body)
            .await
    }
}

/// Request body for getting order list (serializable for POST)
#[derive(Debug, Clone, Serialize)]
struct GetOrderListRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    order_status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    create_time_ge: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    create_time_lt: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    update_time_ge: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    update_time_lt: Option<i64>,
    page_size: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<String>,
}

impl From<GetOrderListRequest> for GetOrderListRequestBody {
    fn from(req: GetOrderListRequest) -> Self {
        Self {
            order_status: req.order_status.map(|s| s.as_code()),
            create_time_ge: req.create_time_ge,
            create_time_lt: req.create_time_lt,
            update_time_ge: req.update_time_ge,
            update_time_lt: req.update_time_lt,
            page_size: req.page_size,
            page_token: req.page_token,
            sort_field: req.sort_field,
            sort_order: req.sort_order,
        }
    }
}

/// Request parameters for getting order list
#[derive(Debug, Clone, Default)]
pub struct GetOrderListRequest {
    /// Filter by order status
    /// Possible values:
    /// - 100: Unpaid
    /// - 111: Awaiting shipment
    /// - 112: Awaiting collection
    /// - 114: Partially shipped
    /// - 121: In transit
    /// - 122: Delivered
    /// - 130: Completed
    /// - 140: Cancelled
    pub order_status: Option<OrderStatus>,

    /// Filter by creation time (Unix timestamp) - greater than or equal
    pub create_time_ge: Option<i64>,

    /// Filter by creation time (Unix timestamp) - less than
    pub create_time_lt: Option<i64>,

    /// Filter by update time (Unix timestamp) - greater than or equal
    pub update_time_ge: Option<i64>,

    /// Filter by update time (Unix timestamp) - less than
    pub update_time_lt: Option<i64>,

    /// Number of orders to return per page (1-50, default: 10)
    pub page_size: i32,

    /// Page token for pagination (from previous response)
    pub page_token: Option<String>,

    /// Field to sort by (e.g., "create_time", "update_time")
    pub sort_field: Option<String>,

    /// Sort order: "ASC" or "DESC"
    pub sort_order: Option<String>,
}

impl GetOrderListRequest {
    pub fn new() -> Self {
        Self {
            page_size: 10,
            ..Default::default()
        }
    }

    pub fn with_status(mut self, status: OrderStatus) -> Self {
        self.order_status = Some(status);
        self
    }

    pub fn with_create_time_range(mut self, start: i64, end: i64) -> Self {
        self.create_time_ge = Some(start);
        self.create_time_lt = Some(end);
        self
    }

    pub fn with_update_time_range(mut self, start: i64, end: i64) -> Self {
        self.update_time_ge = Some(start);
        self.update_time_lt = Some(end);
        self
    }

    pub fn with_page_size(mut self, size: i32) -> Self {
        self.page_size = size.min(50).max(1);
        self
    }

    pub fn with_page_token(mut self, token: String) -> Self {
        self.page_token = Some(token);
        self
    }

    pub fn sort_by(mut self, field: String, order: SortOrder) -> Self {
        self.sort_field = Some(field);
        self.sort_order = Some(order.to_string());
        self
    }
}

/// Order status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    /// 100: Unpaid
    Unpaid,
    /// 111: Awaiting shipment
    AwaitingShipment,
    /// 112: Awaiting collection
    AwaitingCollection,
    /// 114: Partially shipped
    PartiallyShipped,
    /// 121: In transit
    InTransit,
    /// 122: Delivered
    Delivered,
    /// 130: Completed
    Completed,
    /// 140: Cancelled
    Cancelled,
}

impl OrderStatus {
    pub fn as_code(&self) -> i32 {
        match self {
            OrderStatus::Unpaid => 100,
            OrderStatus::AwaitingShipment => 111,
            OrderStatus::AwaitingCollection => 112,
            OrderStatus::PartiallyShipped => 114,
            OrderStatus::InTransit => 121,
            OrderStatus::Delivered => 122,
            OrderStatus::Completed => 130,
            OrderStatus::Cancelled => 140,
        }
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_code())
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "ASC"),
            SortOrder::Descending => write!(f, "DESC"),
        }
    }
}

/// Response from get order list API
#[derive(Debug, Deserialize, Serialize)]
pub struct GetOrderListResponse {
    /// List of orders
    pub orders: Vec<Order>,

    /// Total number of orders matching the filter
    pub total: i64,

    /// Token for next page (if more results available)
    pub next_page_token: Option<String>,

    /// Whether there are more pages
    pub more: bool,
}

/// Order information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Order {
    /// Order ID
    pub id: String,

    /// Order status code
    pub status: i32,

    /// Order creation time (Unix timestamp)
    pub create_time: i64,

    /// Order update time (Unix timestamp)
    pub update_time: i64,

    /// Payment information
    #[serde(default)]
    pub payment: Option<PaymentInfo>,

    /// Recipient information
    #[serde(default)]
    pub recipient_address: Option<RecipientAddress>,

    /// Order items
    #[serde(default)]
    pub item_list: Vec<OrderItem>,

    /// Buyer information
    #[serde(default)]
    pub buyer: Option<BuyerInfo>,

    /// Delivery information
    #[serde(default)]
    pub delivery: Option<DeliveryInfo>,

    /// Split or combine status
    #[serde(default)]
    pub split_or_combine_tag: Option<i32>,

    /// Fulfillment type
    #[serde(default)]
    pub fulfillment_type: Option<i32>,

    /// Warehouse ID
    #[serde(default)]
    pub warehouse_id: Option<String>,

    /// Request cancel time
    #[serde(default)]
    pub request_cancel_time: Option<i64>,

    /// Seller note
    #[serde(default)]
    pub seller_note: Option<String>,

    /// Buyer message
    #[serde(default)]
    pub buyer_message: Option<String>,

    /// Cancel order seller notes
    #[serde(default)]
    pub cancel_order_sn: Option<String>,
}

/// Payment information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaymentInfo {
    /// Currency code (e.g., "USD", "GBP")
    pub currency: String,

    /// Total amount
    pub total_amount: String,

    /// Subtotal
    pub sub_total: String,

    /// Shipping fee
    pub shipping_fee: String,

    /// Seller discount
    pub seller_discount: String,

    /// Platform discount
    pub platform_discount: String,

    /// Tax
    #[serde(default)]
    pub tax: Option<String>,
}

/// Recipient address
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RecipientAddress {
    /// Full address
    #[serde(default)]
    pub full_address: Option<String>,

    /// Recipient name
    #[serde(default)]
    pub name: Option<String>,

    /// Phone number
    #[serde(default)]
    pub phone: Option<String>,

    /// Region code
    #[serde(default)]
    pub region_code: Option<String>,

    /// Postal code
    #[serde(default)]
    pub postal_code: Option<String>,
}

/// Order item
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderItem {
    /// Item ID
    pub id: String,

    /// Product ID
    pub product_id: String,

    /// Product name
    pub product_name: String,

    /// SKU ID
    pub sku_id: String,

    /// SKU name
    #[serde(default)]
    pub sku_name: Option<String>,

    /// SKU image URL
    #[serde(default)]
    pub sku_image: Option<String>,

    /// Quantity
    pub quantity: i32,

    /// Sale price
    pub sale_price: String,

    /// Original price
    #[serde(default)]
    pub original_price: Option<String>,

    /// Seller SKU
    #[serde(default)]
    pub seller_sku: Option<String>,

    /// Platform discount
    #[serde(default)]
    pub platform_discount: Option<String>,

    /// Seller discount
    #[serde(default)]
    pub seller_discount: Option<String>,
}

/// Buyer information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BuyerInfo {
    /// Buyer user ID (encrypted)
    #[serde(default)]
    pub id: Option<String>,

    /// Buyer email (encrypted)
    #[serde(default)]
    pub email: Option<String>,
}

/// Delivery information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeliveryInfo {
    /// Delivery option ID
    #[serde(default)]
    pub delivery_option_id: Option<String>,

    /// Delivery option name
    #[serde(default)]
    pub delivery_option_name: Option<String>,

    /// Shipping provider ID
    #[serde(default)]
    pub shipping_provider_id: Option<String>,

    /// Shipping provider name
    #[serde(default)]
    pub shipping_provider_name: Option<String>,

    /// Tracking number
    #[serde(default)]
    pub tracking_number: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status_codes() {
        assert_eq!(OrderStatus::Unpaid.as_code(), 100);
        assert_eq!(OrderStatus::AwaitingShipment.as_code(), 111);
        assert_eq!(OrderStatus::Completed.as_code(), 130);
    }

    #[test]
    fn test_request_builder() {
        let request = GetOrderListRequest::new()
            .with_status(OrderStatus::AwaitingShipment)
            .with_page_size(20)
            .sort_by("create_time".to_string(), SortOrder::Descending);

        assert_eq!(request.order_status, Some(OrderStatus::AwaitingShipment));
        assert_eq!(request.page_size, 20);
        assert_eq!(request.sort_field, Some("create_time".to_string()));
        assert_eq!(request.sort_order, Some("DESC".to_string()));
    }

    #[test]
    fn test_page_size_limits() {
        let request = GetOrderListRequest::new().with_page_size(100);
        assert_eq!(request.page_size, 50); // Should be capped at 50

        let request = GetOrderListRequest::new().with_page_size(0);
        assert_eq!(request.page_size, 1); // Should be minimum 1
    }
}
