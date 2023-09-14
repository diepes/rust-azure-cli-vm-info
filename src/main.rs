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
        if sub_key == "" {
            print!("{key}, ");
        } else {
            print!("{sub_key}, ");
        }
    }
    println!();
    // print out vms
    for vm in vms.members() {
        let mut vm2 = vm.clone();
        for (key, sub_key) in &print_keys {
            if sub_key == "" {
                print!("{}, ", vm2.remove(&key));
            } else {
                print!("{}, ", vm2.remove(&key)[sub_key]);
            }
        }
        println!();
    }
}
fn enrich_vm_fields(vms: &mut JsonValue, print_keys: &mut Vec<(String, String)>) {
    for vm in vms.members_mut() {
        let new_values = match vm["hardwareProfile"]["vmSize"].as_str() {
            Some("Standard_B8ms") => [
                "8",
                "8vCPU+32GB",
                "Opt.flex.Standard_B1ms (BS High Mem) 1vCPU+2GB",
            ],
            Some("Standard_B2s") => [
                "2",
                "2vCPU+4GB",
                "Opt.flex.Standard_B1ls (BS Series) 1vCPU+0.5GB",
            ],
            Some("Standard_E8s_v3") => [
                "4",
                "8vCPU+64GB",
                "Opt.flex.Standard_E2s_v3 (ESv3) 2vCPU+16GB",
            ],
            Some("Standard_E4s_v3") => [
                "2",
                "4vCPU+32GB",
                "Opt.flex.Standard_E2s_v3 (ESv3) 2vCPU+16GB",
            ],
            Some("Standard_D8s_v3") => [
                "4",
                "8vCPU+32GB",
                "Opt.flex.Standard_D2s_v3 (Dsv3) 2vCPU+8GB",
            ],
            Some("Standard_D4s_v3") => [
                "2",
                "4vCPU+16GB",
                "Opt.flex.Standard_D2s_v3 (Dsv3) 2vCPU+8GB",
            ],
            Some("Standard_D2s_v3") => [
                "1",
                "2vCPU+8GB",
                "Opt.flex.Standard_D2s_v3 (Dsv3) 2vCPU+8GB",
            ],
            _ => ["N.A", "N.A", "N.A"],
        };
        println!(
            "debug {:?} {:?}",
            vm["hardwareProfile"]["vmSize"].as_str(),
            new_values
        );
        for (i, new_key) in ["ReserveUnits", "Cpu+Ram", "ReserveOptFlex"]
            .iter()
            .enumerate()
        {
            let new_key_subkey = (new_key.to_string(), "".to_string());
            vm.insert(new_key, new_values[i])
                .expect("Couldnt insert into JsonValue");
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
