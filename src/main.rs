// cargo watch -x 'fmt' -x 'run'  // 'run -- --some-arg'
// use shutil::pipe; //https://docs.rs/shutil/latest/shutil/fn.pipe.html

//#[macro_use]
// Tried 2023-09-13 hit thread 'main' panic - hosted in gitlab, no way to report issue
// extern crate run_shell; //https://docs.rs/run_shell/latest/run_shell/
// use run_shell::*;

use dotenv;

// https://docs.rs/json/latest/json/
extern crate json;
mod cmd;
mod write_banner;

fn main() {
    dotenv::dotenv().ok();

    let subs = cmd::run("az account list --query '[].name' --output json");
    let az_sub = subs[0].as_str().expect("ERR converting subs[0] to str");
    eprintln!("Got az_sub='{az_sub}'");

    let vms = cmd::run(&format!(
        "az vm list --subscription '{az_sub}' --output json"
    ));

    eprintln!("");
    for vm in vms.members() {
        print!("\"{name}\" >> ", name = vm["name"]);
        for (k, v) in vm.entries() {
            if [
                "name",
                "zones",
                "type",
                "location",
                "provisioningState",
                "resourceGroup",
                "vmId",
                "tags",
            ]
            .contains(&k)
            {
                print!("{k}={v}, ");
            }
        }
        println!("\n");
    }
}
