use regex;
use serde::Deserialize;
use std::error::Error;

use crate::cmd;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")] // camelcase that starts with capital
pub struct AzurePricing {
    pub billing_currency: String,
    pub count: usize,
    pub customer_entity_id: String,
    pub customer_entity_type: String,
    pub items: Vec<Pricing>,
}

#[derive(Debug, Clone, Deserialize)]
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
    //type: String, Consumption, reservation
    pub calc: Option<PricingCalc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PricingCalc {
    pub retail_price_1hr_consumption: f64,
    pub spot_price_1hr_consumption: f64,
    pub retail_price_1year: f64,
    pub retail_price_3year: f64,
    pub count: f64,
    pub flex_base_sku_name: String,
}
impl PricingCalc {
    fn new(
        rp1hr: f64,
        sp1hr: f64,
        rp1y: f64,
        rp3y: f64,
        count: f64,
        flex_base_sku_name: &str,
    ) -> Self {
        PricingCalc {
            retail_price_1hr_consumption: rp1hr,
            spot_price_1hr_consumption: sp1hr,
            retail_price_1year: rp1y,
            retail_price_3year: rp3y,
            count: count,
            flex_base_sku_name: flex_base_sku_name.to_string(),
        }
    }
}

impl Pricing {
    fn new(
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
            PricingCalc::new(0.0, 0.0, 0.0, 0.0, 0.0, "not set")
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
    fn update_price_1hr_consumption(&mut self, price: f64) {
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
    fn update_price_1hr_spot(&mut self, price: f64) {
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
    fn update_price_1year(&mut self, price: f64) {
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

    fn update_price_3year(&mut self, price: f64) {
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

pub fn get_sku_pricing(
    arm_sku_name: &str,
    flex_ratio: f64,
    flex_base_sku_name: &str,
) -> Result<Pricing, Box<dyn Error>> {
    // Query Azure to find match for flex_base_sku_name
    //let arm_sku_name = "Standard_B2s";
    let arm_region_name = "australiaeast";
    // Pricing to return
    let mut p: Pricing = Pricing::new(
        arm_sku_name,
        arm_region_name,
        "USD",
        flex_ratio,
        flex_base_sku_name,
    );
    //#
    // calc/guess sku_name to match from flex_base_sku_name=e.g.('Standard_E2s_v3') => 'E2s v3'
    assert_eq!(flex_base_sku_name[0..9], "Standard_".to_string());
    let guess_sku_name = flex_base_sku_name[9..].replace("_", " ");
    // guess_sku_name = e.g.("E2s v3") calc product name ....ESv3....  drop 2nd char, remove spaces.
    let pattern_product_name = r"Virtual Machines \w+ Series";
    let regex_product_name =
        regex::Regex::new(pattern_product_name).expect("Failed to create regex");

    let mut az_cmd = format!(
        r#"
        az rest --method get
        --url "https://prices.azure.com/api/retail/prices?$filter=armRegionName eq '{arm_region_name}' and armSkuName eq '{flex_base_sku_name}'"
        --output json
        "#
    );
    az_cmd = az_cmd.trim_start().replace("\n", "");
    log::debug!("az_cmd={az_cmd}");
    let data_json = cmd::run(&az_cmd)?;
    log::trace!("data_json={:#?}", data_json);
    let data: AzurePricing = serde_json::from_str(&data_json)?;
    //#
    let mut num_found = 0;
    let mut spot_found = false;
    for (i, price_option) in data.items.iter().enumerate() {
        if price_option.arm_sku_name != flex_base_sku_name {
            log::debug!("i={i} skip sku_name='{}' / arm_sku_name='{}' != flex_sku='{flex_base_sku_name}'  flex_ratio={flex_ratio}", price_option.sku_name, price_option.arm_sku_name);
            continue;
        }
        log::debug!("i={i} got Azure pricing sku_name='{}' == flex_sku='{flex_base_sku_name}' flex_ratio={flex_ratio} {:#?}'", price_option.sku_name, price_option);
        assert_eq!(p.arm_region_name, arm_region_name);
        assert_eq!(price_option.arm_sku_name, flex_base_sku_name);
        // assert_eq!(price_option.arm_sku_name, p.arm_sku_name);
        assert_eq!(p.currency_code, price_option.currency_code);
        assert_eq!("1 Hour", price_option.unit_of_measure);
        assert_eq!(price_option.sku_name, price_option.meter_name);
        assert!(price_option.sku_name.starts_with(&guess_sku_name));

        if price_option.product_name.ends_with("Windows") {
            log::debug!(
                "i={i} Skip pricing for Windows sku_name='{}', product_name='{}'",
                price_option.sku_name,
                price_option.product_name
            );
            continue;
        }
        if !regex_product_name.is_match(&price_option.product_name) {
            log::warn!(
                "i={i} Skip unknown product '{}' looking for regex match '{pattern_product_name}'",
                price_option.product_name
            );
            continue;
        }

        match (
            price_option.type_bill.as_str(),
            &price_option.reservation_term,
            &price_option.sku_name,
        ) {
            ("Consumption", None, sku) if sku == &guess_sku_name => {
                p.update_price_1hr_consumption(price_option.retail_price);
                num_found += 1;
                log::debug!("i={i} num_found={num_found} match #1 pay as you go Consumption");
            }

            ("Consumption", None, sku) if sku == &format!("{} Spot", guess_sku_name) => {
                // We can have normal or spot or something else
                p.update_price_1hr_spot(price_option.retail_price);
                // num_found += 1;  Don't count Spot not available on all sizes
                assert_eq!(spot_found, false, "2nd Spot instance ??");
                spot_found = true;
                log::debug!("i={i}+spot num_found={num_found} match #1 SPOT Consumption");
            }

            ("Consumption", None, sku) if sku == &format!("{} Low Priority", guess_sku_name) => {
                // We can have normal or spot or something else
                log::debug!("i={i} skip 'Low Priority' vm, ToDo add to pricing");
            }

            ("Reservation", Some(value), sku) if sku == &guess_sku_name && value == "1 Year" => {
                p.update_price_1year(price_option.retail_price);
                num_found += 1;
                log::debug!("i={i} num_found={num_found} match 1year Reservation");
            }

            ("Reservation", Some(value), sku) if sku == &guess_sku_name && value == "3 Years" => {
                p.update_price_3year(price_option.retail_price);
                num_found += 1;
                log::debug!("i={i} num_found={num_found} match 3year Reservation");
            }

            (a, b, sku) => {
                log::error!(
                    "i={i} wildcard arm of pricing match should not be reached {:#?} a={a} b={b:?} sku={sku}",
                    price_option
                )
            }
        }
    }
    if num_found != 3 {
        log::error!(
            "We should only find 3 valid prices - Consumption, 1y, 3y, we got {num_found}  spot={spot_found}"
        );
        println!("arm_sku_name={} {}", arm_sku_name, flex_ratio);
        //println!("{:?}", data.items);
        //println!("data={:#?}", data);
        panic!("Wrong number of price's found ! {}", num_found);
    }

    Ok(p)
}
