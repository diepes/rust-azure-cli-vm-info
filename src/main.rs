// cargo watch -x 'fmt' -x 'run'  // 'run -- --some-arg'

use dotenv;
// https://docs.rs/json/latest/json/
extern crate json;
use json::JsonValue;
mod filter_keys;
mod read_csv;

mod cmd;
mod write_banner;
use std::collections::HashMap;
use std::process::ExitCode;

fn main() -> ExitCode {
    dotenv::dotenv().ok();
    // let p = get_sku_pricing(&"Standard_B2s");
    // eprintln!("{p:#?}");
    // if p.arm_region_name.len() > 0 {
    //     panic!("The End");
    // };

    let subs = cmd::run("az account list --query '[].name' --output json");
    let az_sub = subs[0].as_str().expect("ERR getting account list ?");
    eprintln!("Got az_sub='{az_sub}'  subs.len()={}", subs.len());

    // --show-details makes query slower, checks powerstate etc.
    // make it mut so we can drop entries
    let mut vms = cmd::run(&format!(
        "az vm list --subscription '{az_sub}' --show-details --output json"
    ));
    if vms.len() < 1 {
        eprintln!("No vms found in subscription {az_sub} vms:{:#?}", vms);
        return ExitCode::from(1);
    };

    let vm_keys: Vec<String> = vms[0].entries().map(|(k, _)| format!("{}", k)).collect();
    // remove keys for vm.entries()
    for vm in vms.members_mut() {
        let mut skip: String = "".into();
        for key in vm_keys.iter() {
            if filter_keys::remove_key(&key) {
                vm.remove(&key);
                skip += &format!(",\"{key}\"");
            }
        }
    }
    // Add data to VM info, e.g. split Harwareprofile.vmSize into smaller units
    let mut print_keys = filter_keys::get_key_sort_order();

    // now add flex group and ratios for each vm
    enrich_vm_fields(&mut vms, &mut print_keys);

    print_vms(&vms, &print_keys, &az_sub);
    print_summary(&vms);
    return ExitCode::from(0);
}

fn print_vms(vms: &JsonValue, print_keys: &Vec<(String, String)>, az_sub: &str) {
    // 1/2 print csv header
    eprintln!("# VM info CSV, subscription:\"{az_sub}\"");
    for (key, sub_key) in print_keys {
        let print_key;
        if sub_key == "" {
            print_key = escape_csv_field(key);
        } else {
            print_key = escape_csv_field(sub_key);
        }
        print!("{print_key},");
    }
    println!();
    // 2/2 print vms data
    for vm in vms.members() {
        let mut vm2 = vm.clone();
        for (key, sub_key) in print_keys {
            let print_key;
            if sub_key == "" {
                print_key = escape_csv_field(&vm2.remove(&key).to_string());
            } else {
                print_key = escape_csv_field(&vm2.remove(&key)[sub_key].to_string());
            }
            print!("{print_key},");
        }
        println!();
    }
}

fn print_summary(vms: &JsonValue) {
    eprintln!();
    eprintln!();
    eprintln!("# Generate summary of flex servers to reserve to cover all server.");
    let mut summary: HashMap<String, Pricing> = HashMap::new();

    for vm in vms.members() {
        // println!(" summary vm={:?}", vm);
        let name = vm["name"].to_string().clone();
        let vm_size = vm["hardwareProfile"]["vmSize"].to_string().clone();
        assert!(vm_size.len() > 4); //catch empty and null
        let flex_group = vm["flex_group"].to_string().clone();
        let flex_ratio = vm["flex_ratio"].to_string().parse::<f64>().expect(&format!(
            "print_summary flex_ration not a number ? {:?}",
            vm["flex_ratio"]
        ));
        let flex_options = vm["flex_options"].to_string().clone();
        //let flex_option:String = flex_options.split(",")[0].clone();
        // Check if flex_group in summary<HashMap> and update
        if let Some(price) = summary.get(&flex_group) {
            // Clone the retrieved data
            let mut price = price.clone();
            let total_ratio = price.add_ratio_count(flex_ratio);
            // Update it back into the HashMap
            summary.insert(String::from(flex_group.clone()), price);
            println!(
                r#"  Summary update: {} = {} , "{}"={} +{}={} "#,
                name, vm_size, flex_group, flex_options, flex_ratio, total_ratio
            );
        } else {
            println!(
                "  Summary add new name:{name} vm_size:{vm_size} as flex_group:{flex_group} flex_ratio:{flex_ratio} "
            );
            // We have vm and its flex_group, now find pricing for small vm that is 1:1 with flex_group
            let price = get_sku_pricing(&vm_size, flex_ratio, &flex_options);
            eprintln!(
                "   flex_options: {} => {}x{:?}",
                vm_size, flex_ratio, flex_options
            );
            summary.insert(flex_group.clone(), price);
        }
    }
    // print summary
    println!();
    println!("# CSV summary output");
    println!("count,currency,flex_group,price_1hr_consumption,flex_sku,price_1y_consumption,price_1y_save,price_3y_save");
    for (fg, p) in summary.iter() {
        println!(
            r#"{cnt},{cur},{fg},{p0a},"{p0b}",{p0c:.1},{p1},{p3}"#,
            cur = p.currency_code,
            cnt = p.count,
            p0a = p.retail_price_1hr_consumption,
            p0b = p.flex_base_sku_name,
            p0c = p.retail_price_1hr_consumption.parse::<f64>().unwrap() * 24.0 * 365.0,
            p1 = p.retail_price_1year,
            p3 = p.retail_price_3year,
        );
    }
}

#[derive(Debug, Clone)]
pub struct Pricing {
    arm_sku_name: String,
    arm_region_name: String,
    currency_code: String,
    //type: String, Consumption, reservation
    retail_price_1hr_consumption: String,
    retail_price_1year: String,
    retail_price_3year: String,
    pub count: f64,
    flex_base_sku_name: String,
}
impl Pricing {
    fn new(
        sku_name: &str,
        region: &str,
        currency: &str,
        count: f64,
        flex_base_sku_name: &str,
    ) -> Self {
        Self {
            arm_sku_name: String::from(sku_name),
            arm_region_name: String::from(region),
            currency_code: String::from(currency),
            retail_price_1hr_consumption: String::new(),
            retail_price_1year: String::new(),
            retail_price_3year: String::new(),
            count: count,
            flex_base_sku_name: String::from(flex_base_sku_name),
        }
    }

    fn add_ratio_count(&mut self, ratio: f64) -> f64 {
        self.count += ratio;
        self.count
    }
    fn update_price_1hr_consumption(&mut self, price: &str) {
        self.retail_price_1hr_consumption = String::from(price);
    }
    fn update_price_1year(&mut self, price: &str) {
        self.retail_price_1year = String::from(price);
    }
    fn update_price_3year(&mut self, price: &str) {
        self.retail_price_3year = String::from(price);
    }
}

fn get_sku_pricing(arm_sku_name: &str, flex_ratio: f64, flex_base_sku_name: &str) -> Pricing {
    //let arm_sku_name = "Standard_B2s";
    let arm_region_name = "australiaeast";
    let mut p: Pricing = Pricing::new(
        arm_sku_name,
        arm_region_name,
        "USD",
        flex_ratio,
        flex_base_sku_name,
    );
    let mut az_cmd = format!(
        r#"
        az rest --method get
        --url "https://prices.azure.com/api/retail/prices?$filter=armRegionName eq '{arm_region_name}' and armSkuName eq '{flex_base_sku_name}'"
        --output json
        "#
    );
    az_cmd = az_cmd.trim_start().replace("\n", "");
    // eprintln!("az_cmd={az_cmd}");
    let data = cmd::run(&az_cmd);
    // for o in data.members() {
    //     println!("members {o}");
    // }
    let mut num_found = 0;
    for options in data["Items"].members() {
        assert!(p.arm_region_name == options["armRegionName"].to_string());
        //assert!(p.arm_sku_name == options["armSkuName"].to_string());
        assert!(p.flex_base_sku_name == options["armSkuName"].to_string());
        assert!(p.currency_code == options["currencyCode"].to_string());
        assert!("1 Hour" == options["unitOfMeasure"].to_string());
        let p_type = options["type"].to_string().clone();
        let p_term = options["reservationTerm"].to_string().clone();
        match (p_type.as_str(), p_term.as_str()) {
            ("Consumption", "null") => {
                p.update_price_1hr_consumption(&options["retailPrice"].to_string().as_str());
                num_found += 1;
            }
            ("Reservation", "1 Year") => {
                p.update_price_1year(&options["retailPrice"].to_string().as_str());
                num_found += 1;
            }

            ("Reservation", "3 Years") => {
                p.update_price_3year(&options["retailPrice"].to_string().as_str());
                num_found += 1;
            }

            _ => (),
        }
    }
    if num_found < 3 {
        println!("arm_sku_name={} {}", arm_sku_name, flex_ratio);
        println!("{:?}", data["Items"].members());
        println!("data={:#?}", data);
        panic!("No price found !");
    }

    p
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
fn enrich_vm_fields(vms: &mut JsonValue, print_keys: &mut Vec<(String, String)>) {
    // Azure source 2023-09 Instance size flexibility ratios https://aka.ms/isf
    // enrich vm info with flex group and count from csv
    let get_csv = read_csv::get_azure_flex_groups_from_csv();
    // get csv size_vec vm,flex,cnt   and size_flex flex_size and vec of unit 1 options
    let (size_vec, size_flex) = get_csv.get().expect("No csv data ??");
    for vm in vms.members_mut() {
        let vm_size = vm["hardwareProfile"]["vmSize"]
            .as_str()
            .expect("VM with no vmSize ??");

        // search size_vec for match for current vm_size
        let csv_row = &size_vec
            .iter()
            .find(|row| row.flex_sku_name == vm_size)
            .expect("Unknow vm size ?")
            .clone(); // ensure we have our own copy of the matching row.

        // add new keys to vm
        let s = size_flex[&csv_row.flex_group].join(",");
        for new_key in ["flex_group", "flex_sku_name", "flex_ratio", "flex_options"] {
            let new_key_subkey = (new_key.to_string(), "".to_string());
            let new_value = match new_key {
                "flex_group" => &csv_row.flex_group,
                "flex_sku_name" => &csv_row.flex_sku_name,
                "flex_ratio" => &csv_row.flex_ratio,
                "flex_options" => &s,
                _ => panic!("no match to csv struct"),
            };
            vm.insert(new_key, new_value.clone())
                .expect("Couldn't insert into JsonValue");
            if !print_keys.contains(&new_key_subkey) {
                print_keys.push(new_key_subkey);
            }
        }
    }
}
fn _print_json_entries(j: &JsonValue) {
    println!(
        "{}",
        j.entries()
            .into_iter()
            .map(|(k, _v)| format!("\"{}\"", k))
            .collect::<Vec<String>>()
            .join(",\n")
    );
}
