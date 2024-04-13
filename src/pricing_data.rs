use regex;
//use serde::Deserialize;
use std::error::Error;

use crate::az;
use crate::cleanup_vm_name::VmPriceType;
use crate::cmd;

pub fn get_sku_pricing(
    arm_sku_name: &str,
    flex_ratio: f64,
    flex_base_sku_vec: &Vec<String>,
) -> Result<az::pricing::retrieve::Pricing, Box<dyn Error>> {
    // Query Azure to find match for flex_base_sku_name
    //let arm_sku_name = "Standard_B2s";
    let arm_region_name = "australiaeast";
    // Pick last in Vec for sku name. ToDo: loop on err.
    // 2 base flex options, assume cost is the same and pick the last one.
    let flex_base_sku_name = flex_base_sku_vec[flex_base_sku_vec.len() - 1].clone();
    assert_eq!(flex_ratio, 0.0, "Non Zero flex_ratio ? arm_sku_name='{arm_sku_name}' flex_base_sku_name={flex_base_sku_name:?}");
    // Pricing lookup to return
    let mut p = az::pricing::retrieve::Pricing::new(
        arm_sku_name,
        arm_region_name,
        "USD",
        flex_ratio,
        &flex_base_sku_name,
    );
    //#
    // calc/guess sku_name to match from flex_base_sku_name=e.g.('Standard_E2s_v3') => 'E2s v3'
    assert_eq!(
        flex_base_sku_name[0..9],
        "Standard_".to_string(),
        "flex_base_sku_name should start with Standard_"
    );
    //let guess_sku_name = flex_base_sku_name[9..].replace("_", " ");
    let guess_sku_name =
        crate::cleanup_vm_name::split_and_cleanup_name(&flex_base_sku_name)[0].clone(); // remove Standard_
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
    let data: az::pricing::retrieve::AzurePricing = serde_json::from_str(&data_json)
        .expect(&format!("ERR decoding json: data_json='{data_json:#?}'"));
    log::debug!("AzurePricing data={:#?}", data);
    assert!(
        data.prices.len() > 0,
        "Got no pricing info searchign for {flex_base_sku_name}"
    );
    //#
    log::info!(
        "Search arm_sku_name='{arm_sku_name}' flex_base_sku_name='{flex_base_sku_name}' NumOpt={} flex_base_sku_vec='{flex_base_sku_vec:?}'",
        flex_base_sku_vec.len()
    );
    let mut num_found = 0;
    let mut spot_found = false;
    for (i, price_option) in data.prices.iter().enumerate() {
        // if price_option.arm_sku_name != flex_base_sku_name {
        let (po_sku_name, po_sku_name_type) =
            crate::cleanup_vm_name::split_price_type(&price_option.sku_name);
        let po_sku_name_guess =
            crate::cleanup_vm_name::split_and_cleanup_name(&po_sku_name)[0].clone();
        if (price_option.sku_name != flex_base_sku_name) && (po_sku_name_guess != guess_sku_name) {
            log::info!("i={i} skip po.sku_name='{po_s}' / po.arm_sku_name='{po_a}' != flex_base_sku_name='{flex_base_sku_name}' && po.guess={po_g} != guess_sku_name='{guess_sku_name}'  flex_ratio={flex_ratio}"
                        , po_s=price_option.sku_name, po_a=price_option.arm_sku_name
                        , po_g=po_sku_name_guess
                    );
            continue;
        }
        log::debug!("i={i} got Azure pricing po.sku_name='{po_s}' / po.arm_sku_name='{po_a}' == flex_sku='{flex_base_sku_name}' flex_ratio={flex_ratio} {po:#?}'"
                    , po_s=price_option.sku_name
                    , po_a=price_option.arm_sku_name
                    //, po_sku=price_option.sku_name
                    , po=price_option
                );
        assert_eq!(p.arm_region_name, arm_region_name);
        assert_eq!(price_option.arm_sku_name, flex_base_sku_name);
        // assert_eq!(price_option.arm_sku_name, p.arm_sku_name);
        assert_eq!(p.currency_code, price_option.currency_code);
        assert_eq!("1 Hour", price_option.unit_of_measure);
        // assert that sku_name in meter_name
        // to match compound metername, strip post "Post or Low Priority" and possible prefix Standard_
        if price_option.meter_name.contains(" Spot") && price_option.sku_name.contains(" Spot") {
            let sn = price_option.sku_name.strip_suffix(" Spot").unwrap();
            let sn = sn.strip_prefix("Standard_").unwrap_or(&sn);
            let sn = sn.replace("_", " ");
            let mn = price_option.meter_name.strip_suffix(" Spot").unwrap();
            assert!(
                mn.contains(&sn),
                "sku_name='{sn}' not in meter_name='{mn}' after Spot trim guess_sku_name='{guess_sku_name}'"
            )
        } else if price_option.meter_name.contains(" Low Priority")
            && price_option.sku_name.contains(" Low Priority")
        {
            let sn = price_option.sku_name.strip_suffix(" Low Priority").unwrap();
            let sn = sn.strip_prefix("Standard_").unwrap_or(&sn);
            let sn = sn.replace("_", " ");
            let mn = price_option
                .meter_name
                .strip_suffix(" Low Priority")
                .unwrap();
            assert!(
                mn.contains(&sn),
                "sku_name='{sn}' not in meter_name='{mn}' after Low Priority trim"
            )
        } else {
            assert!(
                //meter_name could container multiple e.g. "D11 v2/DS11 v2 Spot"
                price_option.meter_name.contains(&price_option.sku_name)
                    || price_option.meter_name.contains(&guess_sku_name),
                "sku_name='{sn}' not in meter_name='{mn}'",
                sn = price_option.sku_name,
                mn = price_option.meter_name
            );
        }

        let po_sn = &price_option.sku_name;
        let po_sn = po_sn.strip_prefix("Standard_").unwrap_or(&po_sn);
        let po_sn = po_sn.replace("_", " ");
        assert!(
            po_sn.starts_with(&guess_sku_name),
            "sku_name='{sn}' NOT_start_with guess_sku_name='{guess_sku_name}'",
            sn = price_option.sku_name,
        );

        if price_option.product_name.ends_with("Windows") {
            log::debug!(
                "i={i} Skip priceing for Windows sku_name='{}', product_name='{}'",
                price_option.sku_name,
                price_option.product_name
            );
            continue;
        }
        if !regex_product_name.is_match(&price_option.product_name) {
            log::warn!(
                "i={i} Skip unknown product '{}' looking for regex match '{pattern_product_name}' service_name={}",
                price_option.product_name,
                price_option.service_name,
            );
            continue;
        }

        // price_option.sku_name split into po_sku_name_guess, po_sku_name_type
        match (
            price_option.type_bill.as_str(),
            &price_option.reservation_term,
            &po_sku_name_guess,
            &po_sku_name_type,
        ) {
            ("Consumption", None, po_name, VmPriceType::Normal)
                // if (po_name == &guess_sku_name) || (sku == &price_option.sku_name) =>
                if (po_name == &guess_sku_name) =>
            {
                p.update_price_1hr_consumption(price_option.retail_price);
                num_found += 1;
                log::debug!("i={i} num_found={num_found} match #1 pay as you go Consumption");
            }

            ("Consumption", None, po_name, VmPriceType::Spot)
                // if (po_name == &format!("{} Spot", guess_sku_name))
                //    || (po_name == &format!("{} Spot", price_option.sku_name)) =>
                =>
            {
                // We can have normal or spot or something else
                p.update_price_1hr_spot(price_option.retail_price);
                // num_found += 1;  Don't count Spot not available on all sizes
                assert_eq!(spot_found, false, "2nd Spot instance ??");
                spot_found = true;
                log::debug!("i={i}+spot num_found={num_found} match #1 SPOT Consumption");
            }

            ("Consumption", None, po_name, VmPriceType::LowPriority)
                // if (po_name == &format!("{} Low Priority", guess_sku_name))
                //     || (po_name == &format!("{} Low Priority", price_option.sku_name))
                    =>
            {
                // We can have normal or spot or something else
                log::debug!("i={i} skip 'Low Priority' vm, ToDo add to pricing");
            }

            ("Reservation", Some(value), po_name, VmPriceType::Normal)
                if (po_name == &guess_sku_name)
                    && value == "1 Year" =>
            {
                p.update_price_1year(price_option.retail_price);
                num_found += 1;
                log::debug!("i={i} num_found={num_found} match 1year Reservation");
            }

            ("Reservation", Some(value), po_name, VmPriceType::Normal)
                if (po_name == &guess_sku_name)
                    && value == "3 Years" =>
            {
                p.update_price_3year(price_option.retail_price);
                num_found += 1;
                log::debug!("i={i} num_found={num_found} match 3year Reservation");
            }

            (a, b, po_name, po_type) => {
                log::error!(
                    "i={i} wildcard arm of pricing match should not be reached {:#?} a={a} b={b:?} po_name={po_name} po_type={po_type:?}",
                    price_option
                )
            }
        }
    }
    if num_found < 3 {
        log::error!(
            "We should only find 3 valid prices - [Consumption, 1y, 3y], we got {num_found}  spot={spot_found}");
        log::error!("- arm_sku_name='{arm_sku_name}' flex_ratio='{flex_ratio}'");
        log::error!(
            "- flex_base_sku_name='{flex_base_sku_name}' flex_base_sku_vec={flex_base_sku_vec}",
            flex_base_sku_vec = &flex_base_sku_vec.join(", "),
        );
        log::error!("- guess_sku_name='{guess_sku_name}'");
        //println!("{:?}", data.prices);
        //println!("data={:#?}", data);
        panic!("Wrong number of price's found ! {}", num_found);
    }

    Ok(p)
}
