// cargo watch -x 'fmt' -x 'run'  // 'run -- --some-arg'

use dotenv;
use log;

// https://docs.rs/json/latest/json/
// extern crate json;
// use json::JsonValue;
//use serde_json::Value;
use serde_json::{Map, Value};

use itertools::Itertools;
use std::collections::HashMap;

use std::error::Error;
use std::process::ExitCode;

mod az_vms;
mod filter_keys;
mod pricing_data;
mod read_csv;

mod cmd;
mod write_banner;

pub fn run() -> Result<ExitCode, Box<dyn Error>> {
    dotenv::dotenv().ok();
    log::info!("#Start run()");

    let mut vms = az_vms::get_all()?;
    //let mut vms = az_vms::get_fake()?;

    log::info!("got all vm's {}", vms.len());

    log::debug!("add flex group and ratios for each vm");
    enrich_vm_fields(&mut vms);

    //# print_vms(&vms, &print_keys, &az_sub);
    print_summary(&vms)?;
    return Ok(ExitCode::from(0));
}

fn print_summary(vms: &Vec<filter_keys::Vm>) -> Result<(), Box<dyn Error>> {
    eprintln!();
    eprintln!();
    eprintln!("# Generate summary of flex servers to reserve to cover all server.");
    // sum Pricing per flex_group
    let mut summary: HashMap<String, pricing_data::Pricing> = HashMap::new();

    for (i, vm) in vms.iter().enumerate() {
        // println!(" summary vm={:?}", vm);
        let name = &vm.name;
        let vm_size = &vm.hardware_profile.vm_size;
        let vm_power_state = &vm.power_state;
        assert!(vm_size.len() > 4); //catch empty and null
        let vm_flex_lookup = vm.flex_lookup.clone().unwrap();
        let flex_group = vm_flex_lookup.flex_group;
        let flex_ratio = vm_flex_lookup.flex_ratio.parse::<f64>().expect(&format!(
            "print_summary flex_ration not a number ? {:?}",
            vm_flex_lookup.flex_ratio
        ));
        let flex_options = vm_flex_lookup.flex_options;
        // Check if flex_group in summary<HashMap> and update
        let log_msg: &str;
        let log_current_total_flex: f64;
        let flex_add = if vm_power_state == "VM running" {
            flex_ratio
        } else {
            0.0
        };
        if let Some(price) = summary.get_mut(&flex_group) {
            log_current_total_flex = price.add_ratio_count(flex_add);
            log_msg = "Summary update:";
        } else {
            // We have vm and its flex_group, now find pricing for small vm that is 1:1 with flex_group
            let mut price = pricing_data::get_sku_pricing(&vm_size, 0.0, &flex_options)?;
            log_current_total_flex = price.add_ratio_count(flex_add);
            log_msg = "Summary addnew:";
            summary.insert(flex_group.clone(), price);
        }
        log::info!(
            r#"{log_msg}{:2}, {name}, {vm_size}, =flex:,{flex_group}, {flex_ratio}, {flex_add}, total:{log_current_total_flex}, flexOpt:{flex_options}, {vm_power_state}"#,
            i + 1
        );
    }
    // print summary
    println!();
    println!("# CSV summary output");
    println!("count, currency, flex_group           , flex_sku       , 1h_SPOT, 1hr_AsYouGo, 1y_SPOT, 1y_AsYouGo, 1y_Reserv_USD, 3y_Reserv_USD");
    let mut total_count = 0.0;
    for flex_group in summary.keys().sorted() {
        //for (fg, p) in summary.iter().sorted_by_key(|(&key, _)| key) {
        let p = &summary[flex_group];
        let p_calc = p.get_calc_struct_ro();
        println!(
            r#"{cnt:5}, {cur:8}, {flex_group:21}, {flex_sku:15}, {spota}, {pga:11}, {spotb}, {pgb:10.0}, {p1y:13}, {p3y:13}"#,
            cnt = p_calc.count,
            cur = p.currency_code,
            // fg
            flex_sku = p_calc.flex_base_sku_name,
            spota = if p_calc.spot_price_1hr_consumption > 0.0 {
                format!("{:7.3}", p_calc.spot_price_1hr_consumption)
            } else {
                "      -".to_string()
            },
            spotb = if p_calc.spot_price_1hr_consumption > 0.0 {
                format!("{:7.0}", p_calc.spot_price_1hr_consumption * 24.0 * 365.0)
            } else {
                "      -".to_string()
            },
            pga = p_calc.retail_price_1hr_consumption,
            pgb = p_calc.retail_price_1hr_consumption * 24.0 * 365.0,
            p1y = p_calc.retail_price_1year,
            p3y = p_calc.retail_price_3year,
        );
        total_count += p_calc.count;
    }
    println!("total_count={total_count}");
    Ok(())
}

fn escape_csv_field(input: &str) -> String {
    if input.contains(',') || input.contains('"') {
        // If the string contains a comma or double quote, enclose it in double quotes
        // and escape any double quotes within the field.
        // also excel does not like spaces after comma between fields
        let escaped = input.replace("\"", "\"\"");
        format!("\"{}\"", escaped)
    } else {
        // If the string doesn't contain a comma or double quote, no need to enclose it.
        input.to_string()
    }
}

fn enrich_vm_fields(vms: &mut Vec<filter_keys::Vm>) {
    // Azure source 2023-09 Instance size flexibility ratios https://aka.ms/isf
    // enrich vm info with flex group and count from csv
    let get_csv = read_csv::get_azure_flex_groups_from_csv();
    // get csv size_vec vm,flex,cnt   and size_flex flex_size and vec of unit 1 options
    let (size_vec, size_flex) = get_csv.get().expect("No csv data ??");
    for vm in vms {
        let vm_size = vm.hardware_profile.vm_size.clone();

        // search size_vec for match for current vm_size
        let csv_row = size_vec
            .iter()
            .find(|row| row.flex_sku_name == vm_size)
            .expect("Unknow vm size not in csv?")
            .clone(); // ensure we have our own copy of the matching row.

        // add new keys to vm
        let s = size_flex[&csv_row.flex_group].join(",");
        let vm_flex = filter_keys::FlexLookUp {
            flex_group: csv_row.flex_group.to_string(),
            flex_sku_name: csv_row.flex_sku_name.to_string(),
            flex_ratio: csv_row.flex_ratio.to_string(),
            flex_options: s,
        };
        vm.flex_lookup = Some(vm_flex);
    }
}

fn iterate_json_value(json_value: &Value) {
    match json_value {
        Value::Object(map) => {
            for (key, value) in map {
                println!("Key: {}", key);
                iterate_json_value(value);
            }
        }
        Value::Array(arr) => {
            for value in arr {
                iterate_json_value(value);
            }
        }
        Value::String(s) => {
            println!("String Value: {}", s);
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                println!("Integer Value: {}", i);
            } else if let Some(f) = n.as_f64() {
                println!("Float Value: {}", f);
            }
        }
        Value::Bool(b) => {
            println!("Boolean Value: {}", b);
        }
        Value::Null => {
            println!("Null Value");
        }
    }
}
