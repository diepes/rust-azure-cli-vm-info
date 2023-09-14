// cargo watch -x 'fmt' -x 'run'  // 'run -- --some-arg'
// use shutil::pipe; //https://docs.rs/shutil/latest/shutil/fn.pipe.html

//#[macro_use]
// Tried 2023-09-13 hit thread 'main' panic - hosted in gitlab, no way to report issue
// extern crate run_shell; //https://docs.rs/run_shell/latest/run_shell/
// use run_shell::*;

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
    // print csv header
    eprintln!("# VM info CSV, subscription:\"{az_sub}\"");
    for (key, sub_key) in filter_keys::get_key_sort_order() {
        if sub_key == "" {
            print!("{key}, ");
        } else {
            print!("{sub_key}, ");
        }
    }
    // print out vms
    for vm in vms.members() {
        let mut vm2 = vm.clone();
        for (key, sub_key) in filter_keys::get_key_sort_order() {
            if sub_key == "" {
                print!("{}, ", vm2.remove(&key));
            } else {
                print!("{}, ", vm2.remove(&key)[sub_key]);
            }
        }
        println!();
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
