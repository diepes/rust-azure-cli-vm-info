// cargo watch -x 'fmt' -x 'run'  // 'run -- --some-arg'
// use shutil::pipe; //https://docs.rs/shutil/latest/shutil/fn.pipe.html

//#[macro_use]
// Tried 2023-09-13 hit thread 'main' panic - hosted in gitlab, no way to report issue
// extern crate run_shell; //https://docs.rs/run_shell/latest/run_shell/
// use run_shell::*;

// not tried rust-shell by Google
// https://github.com/google/rust-shell

use dotenv;

// https://docs.rs/json/latest/json/
extern crate json;
use json::JsonValue;
mod filter_keys;

mod cmd;
mod write_banner;

fn main() {
    dotenv::dotenv().ok();

    let subs = cmd::run("az account list --query '[].name' --output json");
    let az_sub = subs[0].as_str().expect("ERR converting subs[0] to str");
    eprintln!("Got az_sub='{az_sub}'");

    // --show-details makes query slower, checks powerstate etc.
    // make it mut so we can drop entries
    let mut vms = cmd::run(&format!(
        "az vm list --subscription '{az_sub}' --show-details --output json"
    ));

    eprintln!("");
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
    enrich_vm_fields(&mut vms, &mut print_keys);

    // print csv header
    eprintln!("# VM info CSV, subscription:\"{az_sub}\"");
    for (key, sub_key) in &print_keys {
        let print_key;
        if sub_key == "" {
            print_key = escape_csv_field(key);
        } else {
            print_key = escape_csv_field(sub_key);
        }
        print!("{print_key},");
    }
    println!();
    // print out vms
    for vm in vms.members() {
        let mut vm2 = vm.clone();
        for (key, sub_key) in &print_keys {
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
    let get_csv = filter_keys::get_azure_flex_groups();
    let size_vec = get_csv.get().expect("No csv data ??");
    for vm in vms.members_mut() {
        let vm_size = vm["hardwareProfile"]["vmSize"]
            .as_str()
            .expect("VM with no vmSize ??");

        let csv_row = &size_vec
            .iter()
            .find(|row| row.flex_sku_name == vm_size)
            .expect("Unknow vm size ?")
            .clone(); // ensure we have our own copy of the matching row.

        for new_key in ["flex_group", "flex_sku_name", "flex_ratio"] {
            let new_key_subkey = (new_key.to_string(), "".to_string());
            let new_value = match new_key {
                "flex_group" => &csv_row.flex_group,
                "flex_sku_name" => &csv_row.flex_sku_name,
                "flex_ratio" => &csv_row.flex_ratio,
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
