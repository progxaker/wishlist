use serde_json as json;

use crate::error::Error;
use crate::store::ItemInfo;
use crate::utils;

use serde::Deserialize;
use urlencoding::encode;

pub struct Eva
{
    name: &'static str,
}

#[derive(Deserialize)]
struct StockBlock
{
    is_in_stock: bool,
}

#[derive(Deserialize)]
struct SearchChild
{
    stock: StockBlock,
    name: String,
    #[serde(rename="externalAttr100050")]
    attribute: i64,
    price: f64,
}

impl Eva
{
    pub fn new() -> Self
    {
        Self{ name: "eva-ua" }
    }

    fn dataURL(&self, id: &str) -> String
    {
        let search_api_url = "https://pwa-api.eva.ua/api/catalog/eva_catalog_default/product/_search";
        let source_include_value = "configurable_children.externalAttr100050,configurable_children.price,configurable_children.name,configurable_children.stock.is_in_stock";
        let request_value_raw = format!("{{\"query\":{{\"bool\":{{\"filter\":{{\"terms\":{{\"id\":[{}]}}}}}}}}}}", id);
        let request_value = encode(&request_value_raw);
        format!("{}?_source_include={}&request={}", search_api_url, source_include_value, request_value)
    }

    fn productURL(&self, id: &str, attribute: &str) -> String
    {
        format!("https://eva.ua/ua/{}/#/{}/", id, attribute)
    }

    pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
    {
        let (product_value, attribute_value) = id.split_once("-").ok_or_else(|| {
            rterr!("Failed to parse ID")
        })?;

        let product_id = product_value.strip_prefix("pr").ok_or_else(|| {
            rterr!("Failed to extract product ID")
        })?;

        let dataUrl = self.dataURL(product_id);

        let json_str = utils::get(&dataUrl).await?;

        let productUrl = self.productURL(product_value, attribute_value);

        let data: json::Value = serde_json::from_str(&json_str).map_err(|err| {
            rterr!("Failed to parse JSON: {}", err)
        })?;

        let configurable_children_json = &data["hits"]["hits"][0]["_source"]["configurable_children"];

        let configurable_children: Vec<SearchChild> = serde_json::from_value(
            configurable_children_json.clone()).map_err(|err| {
                rterr!("Failed to parse configurable children: {}", err)
            })?;

        // FIXME: the price can be in JSON even if product isn't present
        let search_result = configurable_children.iter().find(|child|
            child.attribute == attribute_value.parse::<i64>().unwrap_or_default());

        let (is_in_stock, name, price) = match search_result {
            Some(child) => (child.stock.is_in_stock, child.name.clone(), child.price),
            None => (false, String::new(), 0.0),
        };

        if name.is_empty() || price == 0.0 {
            return Err(rterr!("Failed to find product sub-type"));
        }

        let mut item = ItemInfo::new(self.name, id);
        item.name = name.to_owned();
        item.price = (price * 100.0) as i64;
        item.price_str = if is_in_stock { format!("₴{}", price) } else { "Out of stock".to_string() };
        item.url = productUrl;
        Ok(item)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use tokio;

    #[test]
    fn get_price() -> Result<(), Error>
    {
        let a = Eva::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let item = rt.block_on(a.get("pr20285-73278"))?;
        assert!(item.name.find("Garnier Fructis").is_some());
        assert!(item.price > 0);
        Ok(())
    }
}
