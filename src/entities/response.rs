use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Response<T = ()> {
    pub data: Option<T>,
    pub message: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub total_items: Option<u64>,
    pub total_pages: Option<u32>,
}
