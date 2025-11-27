use crate::error::AppError;
use crate::requests::TikTokShopApiClient;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct OrderClient {
    api_client: TikTokShopApiClient,
}

impl OrderClient {
    pub fn new(app_key: String, app_secret: String) -> Self {
        Self {
            api_client: TikTokShopApiClient::new(app_key, app_secret),
        }
    }

    pub async fn get_order_list(
        &self,
        access_token: &str,
        shop_cipher: Option<&str>,
        shop_id: Option<&str>,
        request: GetOrderListRequest,
    ) -> Result<GetOrderListResponse, AppError> {
        // Based on working cURL: body should be empty {}, all params in query string
        let empty_body = serde_json::json!({});

        // Build extra query parameters
        let mut extra_params = BTreeMap::new();
        extra_params.insert("version".to_string(), "202309".to_string());

        if let Some(id) = shop_id {
            extra_params.insert("shop_id".to_string(), id.to_string());
        }

        // Add optional filter parameters to query string
        if let Some(status) = request.order_status {
            extra_params.insert("order_status".to_string(), status.as_code().to_string());
        }
        if let Some(ct_ge) = request.create_time_ge {
            extra_params.insert("create_time_ge".to_string(), ct_ge.to_string());
        }
        if let Some(ct_lt) = request.create_time_lt {
            extra_params.insert("create_time_lt".to_string(), ct_lt.to_string());
        }
        if let Some(ut_ge) = request.update_time_ge {
            extra_params.insert("update_time_ge".to_string(), ut_ge.to_string());
        }
        if let Some(ut_lt) = request.update_time_lt {
            extra_params.insert("update_time_lt".to_string(), ut_lt.to_string());
        }

        extra_params.insert("page_size".to_string(), request.page_size.to_string());

        if let Some(token) = request.page_token {
            extra_params.insert("page_token".to_string(), token);
        }
        if let Some(field) = request.sort_field {
            extra_params.insert("sort_field".to_string(), field);
        }
        if let Some(order) = request.sort_order {
            extra_params.insert("sort_order".to_string(), order);
        }

        // V2 API endpoint: /order/202309/orders/search
        self.api_client
            .post(
                "/order/202309/orders/search",
                Some(access_token),
                shop_cipher,
                &empty_body,
                Some(extra_params),
            )
            .await
    }
}

/// Request parameters for getting order list
#[derive(Debug, Clone, Default)]
pub struct GetOrderListRequest {
    pub order_status: Option<OrderStatus>,
    pub create_time_ge: Option<i64>,
    pub create_time_lt: Option<i64>,
    pub update_time_ge: Option<i64>,
    pub update_time_lt: Option<i64>,
    pub page_size: i32,
    pub page_token: Option<String>,
    pub sort_field: Option<String>,
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
        self.page_size = size.clamp(1, 50);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Unpaid,
    AwaitingShipment,
    AwaitingCollection,
    PartiallyShipped,
    InTransit,
    Delivered,
    Completed,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct GetOrderListResponse {
    pub orders: Vec<Order>,
    #[serde(rename = "total_count")]
    pub total: i64,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Order {
    pub id: String,
    pub status: String,
    pub create_time: i64,
    pub update_time: i64,
    #[serde(default)]
    pub payment: Option<PaymentInfo>,
    #[serde(default)]
    pub recipient_address: Option<RecipientAddress>,
    #[serde(rename = "line_items", default)]
    pub item_list: Vec<OrderItem>,
    #[serde(default)]
    pub fulfillment_type: Option<String>,
    #[serde(default)]
    pub warehouse_id: Option<String>,
    #[serde(default)]
    pub buyer_message: Option<String>,
    #[serde(default)]
    pub buyer_email: Option<String>,
    #[serde(default)]
    pub cancel_order_sla_time: Option<i64>,
    #[serde(default)]
    pub cancel_reason: Option<String>,
    #[serde(default)]
    pub cancel_time: Option<i64>,
    #[serde(default)]
    pub cancellation_initiator: Option<String>,
    #[serde(default)]
    pub collection_due_time: Option<i64>,
    #[serde(default)]
    pub commerce_platform: Option<String>,
    #[serde(default)]
    pub delivery_option_id: Option<String>,
    #[serde(default)]
    pub delivery_option_name: Option<String>,
    #[serde(default)]
    pub delivery_type: Option<String>,
    #[serde(default)]
    pub has_updated_recipient_address: Option<bool>,
    #[serde(default)]
    pub is_cod: Option<bool>,
    #[serde(default)]
    pub is_on_hold_order: Option<bool>,
    #[serde(default)]
    pub is_replacement_order: Option<bool>,
    #[serde(default)]
    pub is_sample_order: Option<bool>,
    #[serde(default)]
    pub order_type: Option<String>,
    #[serde(default)]
    pub packages: Vec<Package>,
    #[serde(default)]
    pub paid_time: Option<i64>,
    #[serde(default)]
    pub payment_method_name: Option<String>,
    #[serde(default)]
    pub rts_sla_time: Option<i64>,
    #[serde(default)]
    pub rts_time: Option<i64>,
    #[serde(default)]
    pub shipping_due_time: Option<i64>,
    #[serde(default)]
    pub shipping_provider: Option<String>,
    #[serde(default)]
    pub shipping_provider_id: Option<String>,
    #[serde(default)]
    pub shipping_type: Option<String>,
    #[serde(default)]
    pub tracking_number: Option<String>,
    #[serde(default)]
    pub tts_sla_time: Option<i64>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub collection_time: Option<i64>,
    #[serde(default)]
    pub delivery_time: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Package {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaymentInfo {
    pub currency: String,
    pub total_amount: String,
    pub sub_total: String,
    pub shipping_fee: String,
    pub seller_discount: String,
    pub platform_discount: String,
    #[serde(default)]
    pub tax: Option<String>,
    #[serde(default)]
    pub original_shipping_fee: Option<String>,
    #[serde(default)]
    pub original_total_product_price: Option<String>,
    #[serde(default)]
    pub shipping_fee_cofunded_discount: Option<String>,
    #[serde(default)]
    pub shipping_fee_platform_discount: Option<String>,
    #[serde(default)]
    pub shipping_fee_seller_discount: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RecipientAddress {
    #[serde(default)]
    pub full_address: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "phone_number", default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub region_code: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub address_detail: Option<String>,
    #[serde(default)]
    pub address_line1: Option<String>,
    #[serde(default)]
    pub address_line2: Option<String>,
    #[serde(default)]
    pub address_line3: Option<String>,
    #[serde(default)]
    pub address_line4: Option<String>,
    #[serde(default)]
    pub district_info: Vec<DistrictInfo>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub first_name_local_script: Option<String>,
    #[serde(default)]
    pub last_name_local_script: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DistrictInfo {
    pub address_level: String,
    pub address_level_name: String,
    pub address_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderItem {
    pub id: String,
    pub product_id: String,
    pub product_name: String,
    pub sku_id: String,
    #[serde(default)]
    pub sku_name: Option<String>,
    #[serde(default)]
    pub sku_image: Option<String>,
    #[serde(default)]
    pub quantity: Option<i32>,
    pub sale_price: String,
    #[serde(default)]
    pub original_price: Option<String>,
    #[serde(default)]
    pub seller_sku: Option<String>,
    #[serde(default)]
    pub platform_discount: Option<String>,
    #[serde(default)]
    pub seller_discount: Option<String>,
    #[serde(default)]
    pub cancel_reason: Option<String>,
    #[serde(default)]
    pub cancel_user: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub display_status: Option<String>,
    #[serde(default)]
    pub gift_retail_price: Option<String>,
    #[serde(default)]
    pub is_gift: Option<bool>,
    #[serde(default)]
    pub package_id: Option<String>,
    #[serde(default)]
    pub package_status: Option<String>,
    #[serde(default)]
    pub rts_time: Option<i64>,
    #[serde(default)]
    pub shipping_provider_id: Option<String>,
    #[serde(default)]
    pub shipping_provider_name: Option<String>,
    #[serde(default)]
    pub sku_type: Option<String>,
    #[serde(default)]
    pub tracking_number: Option<String>,
}
