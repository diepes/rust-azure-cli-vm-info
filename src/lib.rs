// cargo watch -x 'fmt' -x 'run'  // 'run -- --some-arg'

// use dotenv;
use log;

use serde_json::Value;

use itertools::Itertools;
use std::collections::HashMap;

use std::error::Error;
// use std::process::ExitCode;

pub mod az; // subscritpions & vmlist
            // pub mod az_vms;
mod cleanup_vm_name;
mod filter_keys;
mod pricing_data;
mod read_csv;

mod cmd;
mod write_banner;

pub fn print_summary(vms: &az::vmlist::VirtualMachines) -> Result<(), Box<dyn Error>> {
    eprintln!();
    eprintln!();
    eprintln!("# Generate summary of flex servers to reserve to cover all server.");
    // sum Pricing per flex_group
    let mut summary: HashMap<String, az::pricing::retrieve::Pricing> = HashMap::new();

    for (i, vm) in vms.0.iter().enumerate() {
        log::info!(" vm.flex_lookup={:?}", vm.flex_lookup);
        let name = &vm.name;
        let vm_size = &vm.vm_size;
        // let vm_power_state = &vm.power_state;
        let vm_power_state = "VM running";
        assert!(vm_size.len() > 4); //catch empty and null
        let vm_flex_lookup = vm
            .flex_lookup
            .clone()
            .expect(&format!("No flex_lookup data for vm:{name} ?"));
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
        // see if we already have this flex_group lookup.
        if let Some(price) = summary.get_mut(&flex_group) {
            log_current_total_flex = price.add_ratio_count(flex_add);
            log_msg = "Summary update:";
        } else {
            // We have vm and its flex_group, now find pricing for small vm that is 1:1 with flex_group
            let mut price = pricing_data::get_sku_pricing(&vm_size, 0.0, &flex_options)?;
            log_current_total_flex = price.add_ratio_count(flex_add);
            log_msg = "Summary add new:";
            summary.insert(flex_group.clone(), price);
        }
        log::info!(
            r#"{log_msg}{num:2}, {name}, {vm_size}, =flex:,{flex_group}, {flex_ratio}, {flex_add}, total:{log_current_total_flex}, flexOpt:{flex_options}, {vm_power_state}"#,
            num = i + 1,
            flex_options = flex_options.join(", "),
        );
    }
    // print summary
    println!();
    println!("# CSV summary output");
    println!("count, currency, flex_group             , flex_sku         , 1h_SPOT, 1hr_Linux, 1hr_Win, 1ySpLin, 1ySpWin, 1y_Linux, 1y_Win, 1y_Reserv_USD, 3y_Reserv_USD");
    let mut total_count = 0.0;
    for flex_group in summary.keys().sorted() {
        //for (fg, p) in summary.iter().sorted_by_key(|(&key, _)| key) {
        let p = &summary[flex_group];
        let p_calc = p.get_calc_struct_ro();
        println!(
            r#"{cnt:5}, {cur:8}, {flex_group:23}, {flex_sku:17}, {spotl}, {pga:9}, {pgw:7}, {spotly}, {spotwy}, {pgay:9.0} {pgwy:6.0}, {p1y:13}, {p3y:13}"#,
            cnt = p_calc.count,
            cur = p.currency_code,
            // fg
            flex_sku = p_calc.flex_base_sku_name,
            pga = p_calc.retail_price_1hr_consumption,
            pgay = p_calc.retail_price_1hr_consumption * 24.0 * 365.0,
            pgw = p_calc.retail_price_1hr_consumption_windows,
            pgwy = p_calc.retail_price_1hr_consumption_windows * 24.0 * 365.0,
            p1y = p_calc.retail_price_1year,
            p3y = p_calc.retail_price_3year,
            // spot Linux & Win optional
            spotl = if p_calc.spot_price_1hr_consumption > 0.0 {
                format!("{:7.3}", p_calc.spot_price_1hr_consumption)
            } else {
                "      -".to_string()
            },
            spotly = if p_calc.spot_price_1hr_consumption > 0.0 {
                format!("{:7.0}", p_calc.spot_price_1hr_consumption * 24.0 * 365.0)
            } else {
                "      -".to_string()
            },
            // spotw = if p_calc.spot_price_1hr_consumption_windows > 0.0 {
            //     format!("{:7.3}", p_calc.spot_price_1hr_consumption_windows)
            // } else {
            //     "      -".to_string()
            // },
            spotwy = if p_calc.spot_price_1hr_consumption_windows > 0.0 {
                format!(
                    "{:7.0}",
                    p_calc.spot_price_1hr_consumption_windows * 24.0 * 365.0
                )
            } else {
                "      -".to_string()
            },
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

pub fn enrich_vm_fields(vms: &mut az::vmlist::VirtualMachines) {
    // Azure source 2023-09 Instance size flexibility ratios https://aka.ms/isf
    // enrich vm info with flex group and count from csv
    let get_csv = read_csv::get_azure_flex_groups_from_csv();
    // get csv size_vec vm,flex,cnt   and size_flex flex_size and vec of unit 1 options
    let (size_vec, size_flex) = get_csv.get().expect("No csv data ??");
    for vm in &mut vms.0 {
        let vm_size = vm.vm_size.clone();

        // search size_vec for match for current vm_size
        let csv_row = size_vec
            .iter()
            .find(|row| row.flex_sku_name == vm_size)
            .expect(&format!("Unknow vm_size='{}' not in csv?", vm_size));

        // add new keys to vm
        //let s = size_flex[&csv_row.flex_group].join(",");
        let vm_flex = az::vmlist::FlexLookUp {
            flex_group: csv_row.flex_group.to_string(),
            flex_sku_name: csv_row.flex_sku_name.to_string(),
            flex_ratio: csv_row.flex_ratio.to_string(),
            flex_options: size_flex[&csv_row.flex_group].clone(),
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
