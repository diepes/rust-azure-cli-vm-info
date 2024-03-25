use crate::cmd;
use crate::filter_keys::Vm; // Vm struct and parsing
use colored::Colorize;
use std::fs::{self, File};
use std::io::Write;

pub fn get_fake() -> Result<Vec<Vm>, Box<dyn std::error::Error>> {
    let file_names = [
        "src/tests/test_data/az_vm_output_01.txt",
        "src/tests/test_data/az_vm_output_02.txt",
    ];
    // Vector to store Value from each file
    let mut vms_all: Vec<Vm> = vec![];
    // Read JSON files and parse content
    for &f_name in file_names.iter() {
        let file_content = fs::read_to_string(f_name).expect("Failed to read JSON file content");
        let mut vm_vec: Vec<Vm> =
            Vm::from_json_array(&file_content).expect("Failed to parse JSON content");
        vms_all.append(&mut vm_vec);
    }
    Ok(vms_all)
}

pub fn get_all() -> Result<Vec<Vm>, Box<dyn std::error::Error>> {
    //# let mut vms_all = json::JsonValue::Array(Vec::new());
    let mut vms_all: Vec<Vm> = Vec::new();
    // get json Value's
    let subs_get = cmd::run("az account list --query '[].name' --output json")?;
    let subs = cmd::string_to_json_vec_string(&subs_get)?;

    let subs_string: String = subs
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<String>>()
        .join(", ");
    //let subs_string: String = subs_formatted.join(", ");
    log::debug!("loop through subscriptions {}", subs_string);
    //
    for az_sub in subs.into_iter() {
        log::info!("az_sub='{az_sub}'", az_sub = az_sub.on_green());

        // --show-details makes query slower, checks powerstate etc.
        // make it mut so we can drop entries
        let vms_cmd_output = cmd::run(&format!(
            "az vm list --subscription '{az_sub}' --show-details --output json"
        ));
        if let Ok(vms_string) = vms_cmd_output {
            let from_json_result = Vm::from_json_array(&vms_string);
            if let Ok(vms) = from_json_result {
                log::info!("found {num} vm's in subscription {az_sub}", num = vms.len());
                vms_all.extend(vms);
            } else if let Err(err) = from_json_result {
                let file_unparsable_json = "log/unparsable_json.json";
                let mut output = File::create(file_unparsable_json)?;
                write!(output, "{}", vms_string);
                log::error!(
                    "Failed to parse response for subscription {az_sub} ERR:{}\n see json_string in {}",
                    err,
                    file_unparsable_json
                );
                // if let Some(position) = err.line_and_col() {
                //     log::error!("Error occurred at path: {:?}", path);
                // }
                panic!()
            }
        } else {
            log::warn!("No vms found in subscription {az_sub}");
        }
    }
    if vms_all.len() < 1 {
        return Err("No vms found.".into());
    };

    Ok(vms_all)
}

// Define a module for tests
#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_get_vms() {
        let vms_all_result = get_all();
        assert!(vms_all_result.is_ok());
        let vms_all = vms_all_result.unwrap();
        println!("VMS.len ==>> {:#?}", vms_all.len());
        assert!(vms_all.len() > 0, "Expected to find more than one VM");
    }
    // #[test]
    // fn test_get_fake() {
    //     let vms_all_result = get_fake();
    //     assert!(vms_all_result.is_ok());
    //     let vms_all = vms_all_result.unwrap();
    //     assert_eq!(vms_all.len(), 31);
    //     assert!(vms_all[0].is_object(), "Expected to find VM json object");
    //     // verify the last vm has name fake-vm-02
    //     assert_eq!(
    //         vms_all[30]["name"],
    //         serde_json::from_str::<serde_json::Value>(r#""fake-vm-02""#).unwrap()
    //     );
    // }
}
