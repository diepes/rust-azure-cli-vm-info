use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")] // camelcase that starts with capital
pub struct AzurePricing {
    pub billing_currency: String,
    pub count: usize,
    pub customer_entity_id: String,
    pub customer_entity_type: String,
    #[serde(rename = "Items")]
    pub prices: Vec<Pricing>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")] //camelcase that starts with lowercase
pub struct Pricing {
    pub arm_region_name: String,
    pub arm_sku_name: String,
    pub currency_code: String,
    pub location: String,
    pub product_name: String,
    pub sku_name: String,
    pub meter_name: String,
    pub tier_minimum_units: f64,
    pub retail_price: f64,
    pub reservation_term: Option<String>, // only set for type_bill Reservation, not Consumption
    #[serde(rename = "type")]
    pub type_bill: String,
    pub unit_of_measure: String,
    pub unit_price: f64,
    pub service_name: String,
    //type: String, Consumption, reservation
    pub calc: Option<PricingCalc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PricingCalc {
    pub retail_price_1hr_consumption: f64,
    pub retail_price_1hr_consumption_windows: f64,
    pub spot_price_1hr_consumption: f64,
    pub spot_price_1hr_consumption_windows: f64,
    pub retail_price_1year: f64,
    pub retail_price_3year: f64,
    pub count: f64,
    pub flex_base_sku_name: String,
}
impl PricingCalc {
    pub fn new(
        rp1hr: f64,
        rp1hrwin: f64,
        sp1hr: f64,
        sp1hrwin: f64,
        rp1y: f64,
        rp3y: f64,
        count: f64,
        flex_base_sku_name: &str,
    ) -> Self {
        PricingCalc {
            retail_price_1hr_consumption: rp1hr,
            retail_price_1hr_consumption_windows: rp1hrwin,
            spot_price_1hr_consumption: sp1hr,
            spot_price_1hr_consumption_windows: sp1hrwin,
            retail_price_1year: rp1y,
            retail_price_3year: rp3y,
            count: count,
            flex_base_sku_name: flex_base_sku_name.to_string(),
        }
    }
}

impl Pricing {
    pub fn new(
        sku_name: &str,
        region: &str,
        currency: &str,
        flex_count: f64,
        flex_base_sku_name: &str,
    ) -> Self {
        let mut new_pricing = Self {
            arm_sku_name: String::from(sku_name),
            arm_region_name: String::from(region),
            currency_code: String::from(currency),
            calc: None,
            location: "not set".to_string(),
            product_name: "not set".to_string(),
            sku_name: "not set".to_string(),
            meter_name: "not set".to_string(),
            service_name: "not set".to_string(),
            tier_minimum_units: 0.0,
            retail_price: 0.0,
            reservation_term: None,
            type_bill: "not set".to_string(),
            unit_of_measure: "not set".to_string(),
            unit_price: 0.0,
        };
        new_pricing.add_calc_flex_base_sku_name(flex_base_sku_name);
        new_pricing.add_ratio_count(flex_count);
        new_pricing
    }
    pub fn get_calc_struct(&mut self) -> &mut PricingCalc {
        self.calc.get_or_insert_with(|| {
            log::debug!("initialize PricingCalc to zero default");
            PricingCalc::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, "not set")
        })
    }
    pub fn get_calc_struct_ro(&self) -> &PricingCalc {
        match self.calc {
            None => {
                log::error!("called to read PricingCalc but it has not been initialised yet");
                panic!()
            }
            Some(_) => &self.calc.as_ref().unwrap(),
        }
    }

    fn add_calc_flex_base_sku_name(&mut self, flex_base_sku_name: &str) {
        let mut some_calc = self.get_calc_struct();
        some_calc.flex_base_sku_name = flex_base_sku_name.to_string();
    }

    pub fn add_ratio_count(&mut self, ratio: f64) -> f64 {
        let mut some_calc = self.get_calc_struct();
        some_calc.count += ratio;
        some_calc.count
    }
    pub fn update_price_1hr_consumption(&mut self, price: f64) {
        let mut some_calc = self.get_calc_struct();
        let current_price = some_calc.retail_price_1hr_consumption.clone();
        some_calc.retail_price_1hr_consumption = price;
        if current_price == 0.0 {
            log::debug!("set price 1hr {} with {price}", self.arm_sku_name);
        } else {
            log::warn!(
                "update price 1hr {} from {current_price} to {price}",
                self.arm_sku_name
            );
        }
    }
    pub fn update_price_1hr_consumption_windows(&mut self, price: f64) {
        let mut some_calc = self.get_calc_struct();
        let current_price = some_calc.retail_price_1hr_consumption_windows.clone();
        some_calc.retail_price_1hr_consumption_windows = price;
        if current_price == 0.0 {
            log::debug!("set price 1hr WIN {} with {price}", self.arm_sku_name);
        } else {
            log::warn!(
                "update price 1hr WIN {} from {current_price} to {price}",
                self.arm_sku_name
            );
        }
    }
    pub fn update_price_1hr_spot(&mut self, price: f64) {
        let mut some_calc = self.get_calc_struct();
        let current_price = some_calc.spot_price_1hr_consumption.clone();
        some_calc.spot_price_1hr_consumption = price;
        if current_price == 0.0 {
            log::debug!("set spot price 1hr {} with {price}", self.arm_sku_name);
        } else {
            log::warn!(
                "update spot price 1hr {} from {current_price} to {price}",
                self.arm_sku_name
            );
        }
    }
    pub fn update_price_1hr_spot_windows(&mut self, price: f64) {
        let mut some_calc = self.get_calc_struct();
        let current_price = some_calc.spot_price_1hr_consumption_windows.clone();
        some_calc.spot_price_1hr_consumption_windows = price;
        if current_price == 0.0 {
            log::debug!("set spot price 1hr {} with {price}", self.arm_sku_name);
        } else {
            log::warn!(
                "update spot price 1hr {} from {current_price} to {price}",
                self.arm_sku_name
            );
        }
    }
    pub fn update_price_1year(&mut self, price: f64) {
        let mut some_calc = self.get_calc_struct();
        let current_price = some_calc.retail_price_1year.clone();
        some_calc.retail_price_1year = price;
        if current_price == 0.0 {
            log::debug!("set price 1y {} with {price}", self.arm_sku_name);
        } else {
            log::warn!(
                "update price 1y {} from {current_price} to {price}",
                self.arm_sku_name
            );
        }
    }

    pub fn update_price_3year(&mut self, price: f64) {
        let mut some_calc = self.get_calc_struct();
        let current_price = some_calc.retail_price_3year.clone();
        some_calc.retail_price_3year = price;
        if current_price == 0.0 {
            log::debug!("set price 3y {} with {price}", self.arm_sku_name);
        } else {
            log::warn!(
                "update price 3y {} from {current_price} to {price}",
                self.arm_sku_name
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // #[tokio::test]
    fn test_get_prices() {
        let data_json="{\n  \"BillingCurrency\": \"USD\",\n  \"Count\": 5,\n  \"CustomerEntityId\": \"Default\",\n  \"CustomerEntityType\": \"Retail\",
            \"Items\": [
                {\n  \"armRegionName\": \"australiaeast\",\n  \"armSkuName\": \"Standard_B1ls\",
                \"currencyCode\": \"USD\",\n   \"effectiveStartDate\": \"2021-11-01T00:00:00Z\",\n  \"isPrimaryMeterRegion\": true,
                \"location\": \"AU East\",\n      \"meterId\": \"333e3ab2-8ad4-4f1f-aff6-2da9771dfb3e\",\n 
                \"meterName\": \"B1ls\",\n      \"productId\": \"DZH318Z0BQNH\",\n
                \"productName\": \"Virtual Machines BS Series Windows\",\n 
                \"retailPrice\": 0.0106,\n    \"serviceFamily\": \"Compute\",\n   \"serviceId\": \"DZH313Z7MMC8\",
                \"serviceName\": \"Virtual Machines\",\n      \"skuId\": \"DZH318Z0BQNH/00MV\",
                \"skuName\": \"B1ls\",\n      \"tierMinimumUnits\": 0.0,
                \"type\": \"Consumption\",\n      \"unitOfMeasure\": \"1 Hour\",\n  \"unitPrice\": 0.0106\n
                },
                {\n  \"armRegionName\": \"australiaeast\",\n      \"armSkuName\": \"Standard_B1ls\",\n      \"currencyCode\": \"USD\",\n      \"effectiveStartDate\": \"2021-11-01T00:00:00Z\",\n      \"isPrimaryMeterRegion\": true,\n      \"location\": \"AU East\",\n      \"meterId\": \"333e3ab2-8ad4-4f1f-aff6-2da9771dfb3e\",\n      \"meterName\": \"B1ls\",\n      \"productId\": \"DZH318Z0BQNH\",\n
                \"productName\": \"Virtual Machines BS Series Windows\",\n      \"retailPrice\": 0.0066,\n      \"serviceFamily\": \"Compute\",\n      \"serviceId\": \"DZH313Z7MMC8\",\n      \"serviceName\": \"Virtual Machines\",\n      \"skuId\": \"DZH318Z0BQNH/00MV\",\n      \"skuName\": \"B1ls\",\n      \"tierMinimumUnits\": 0.0,\n
                \"type\": \"DevTestConsumption\",\n      \"unitOfMeasure\": \"1 Hour\",\n      \"unitPrice\": 0.0066\n
                },\n
                {\n      \"armRegionName\": \"australiaeast\",\n      \"armSkuName\": \"Standard_B1ls\",\n      \"currencyCode\": \"USD\",\n      \"effectiveStartDate\": \"2021-11-01T00:00:00Z\",\n      \"isPrimaryMeterRegion\": true,\n      \"location\": \"AU East\",\n      \"meterId\": \"ffa460ed-6d90-4b29-97a4-11d8267e6be6\",\n      \"meterName\": \"B1ls\",\n      \"productId\": \"DZH318Z0BQ35\",\n
                \"productName\": \"Virtual Machines BS Series\",\n      \"retailPrice\": 0.0066,\n      \"serviceFamily\": \"Compute\",\n      \"serviceId\": \"DZH313Z7MMC8\",\n      \"serviceName\": \"Virtual Machines\",\n      \"skuId\": \"DZH318Z0BQ35/01BP\",\n      \"skuName\": \"B1ls\",\n      \"tierMinimumUnits\": 0.0,\n
                \"type\": \"Consumption\",\n      \"unitOfMeasure\": \"1 Hour\",\n      \"unitPrice\": 0.0066\n
                },\n
                {\n      \"armRegionName\": \"australiaeast\",\n      \"armSkuName\": \"Standard_B1ls\",\n      \"currencyCode\": \"USD\",\n      \"effectiveStartDate\": \"2018-04-18T00:00:00Z\",\n      \"isPrimaryMeterRegion\": true,\n      \"location\": \"AU East\",\n      \"meterId\": \"ffa460ed-6d90-4b29-97a4-11d8267e6be6\",\n      \"meterName\": \"B1ls\",\n      \"productId\": \"DZH318Z0BQ35\",\n      \"productName\": \"Virtual Machines BS Series\",\n
                \"reservationTerm\": \"1 Year\",\n      \"retailPrice\": 34.0,\n      \"serviceFamily\": \"Compute\",\n      \"serviceId\": \"DZH313Z7MMC8\",\n      \"serviceName\": \"Virtual Machines\",\n      \"skuId\": \"DZH318Z0BQ35/01BN\",\n      \"skuName\": \"B1ls\",\n      \"tierMinimumUnits\": 0.0,\n
                \"type\": \"Reservation\",\n      \"unitOfMeasure\": \"1 Hour\",\n      \"unitPrice\": 34.0\n
                },
                {\n      \"armRegionName\": \"australiaeast\",\n      \"armSkuName\": \"Standard_B1ls\",\n      \"currencyCode\": \"USD\",\n      \"effectiveStartDate\": \"2018-04-18T00:00:00Z\",\n      \"isPrimaryMeterRegion\": true,\n      \"location\": \"AU East\",\n      \"meterId\": \"ffa460ed-6d90-4b29-97a4-11d8267e6be6\",\n      \"meterName\": \"B1ls\",\n      \"productId\": \"DZH318Z0BQ35\",\n      \"productName\": \"Virtual Machines BS Series\",\n
                \"reservationTerm\": \"3 Years\",\n      \"retailPrice\": 65.0,\n      \"serviceFamily\": \"Compute\",\n      \"serviceId\": \"DZH313Z7MMC8\",\n      \"serviceName\": \"Virtual Machines\",\n      \"skuId\": \"DZH318Z0BQ35/01BM\",\n      \"skuName\": \"B1ls\",\n      \"tierMinimumUnits\": 0.0,\n
                \"type\": \"Reservation\",\n      \"unitOfMeasure\": \"1 Hour\",\n      \"unitPrice\": 65.0\n
                }\n  
                ],\n  \"NextPageLink\": null\n}\n";
        let data = serde_json::from_str(&data_json);
        assert!(data.is_ok(), "Err decoding test data_json ?");
        let data: AzurePricing = data.unwrap();
        assert_eq!(
            data.prices[0].product_name,
            "Virtual Machines BS Series Windows"
        );
        assert_eq!(data.prices[0].arm_sku_name, "Standard_B1ls");
        assert_eq!(data.prices[0].sku_name, "B1ls");
        assert_eq!(data.prices[0].retail_price, 0.0106);
        assert_eq!(data.prices[0].type_bill, "Consumption");
        assert_eq!(data.prices[0].reservation_term, None);
        // 1 Year reservation
        assert_eq!(data.prices[3].type_bill, "Reservation");
        assert_eq!(data.prices[3].reservation_term, Some("1 Year".to_string()));
        assert_eq!(data.prices[3].retail_price, 34.0);
        assert_eq!(data.prices[3].unit_price, 34.0);
        // test std values.
    }
}
