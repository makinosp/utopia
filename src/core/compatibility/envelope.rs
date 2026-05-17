use serde::Serialize;

use crate::core::compatibility::pagination::{compute_pagination, Paginated, PaginationMeta};

#[derive(Debug, Clone, Serialize)]
pub struct FireflyListMeta {
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflyListEnvelope<T>
where
    T: Serialize,
{
    pub data: Vec<T>,
    pub meta: FireflyListMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireflySingleEnvelope<T>
where
    T: Serialize,
{
    pub data: T,
}

impl<T> FireflyListEnvelope<T>
where
    T: Serialize + Clone,
{
    pub fn from_paginated(paginated: Paginated<T>) -> Self {
        let meta = compute_pagination(&paginated);

        Self {
            data: paginated.records,
            meta: FireflyListMeta { pagination: meta },
        }
    }
}
