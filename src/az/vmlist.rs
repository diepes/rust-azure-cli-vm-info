/*
USE the AZURE auto gen code

cargo add azure_mgmt_compute
cargo add azure_mgmt_subscription
*/

use azure_identity::AzureCliCredential;
use futures::stream::StreamExt;
//use serde::Serialize;
// use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{ser::Serializer, Serialize};

use std::sync::Arc;

#[derive(Debug)]
pub struct VirtualMachines(pub Vec<VM>);

#[derive(Debug)]
pub struct VM {
    pub id: String,
    pub vm_id: String, //dup id
    pub name: String,
    pub location: String,
    pub zones: Option<Vec<String>>,
    pub flex_lookup: Option<FlexLookUp>, // Added from csv lookup of flex options.
    pub vm_size: String,
    // pub power_state: String,
    pub az: azure_mgmt_compute::models::VirtualMachine, // Value from Rust Azure generated api
}
// needs Clone so we can make a copy to read Some().fields
#[derive(Debug, Clone)]
pub struct FlexLookUp {
    pub flex_group: String,
    pub flex_sku_name: String,
    pub flex_ratio: String,
    pub flex_options: Vec<String>,
}
impl VM {
    pub fn new(
        az_vm: azure_mgmt_compute::models::VirtualMachine,
    ) -> Result<VM, Box<dyn std::error::Error>> {
        Ok(VM {
            id: az_vm.resource.id.clone().expect("resource.id"),
            vm_id: az_vm.resource.id.clone().unwrap(),
            name: az_vm.resource.name.clone().expect("resource.name"),
            location: az_vm.resource.location.clone(),
            zones: Some(az_vm.zones.clone()),
            flex_lookup: None,
            // power_state: az_vm.properties.clone().unwrap().
            vm_size: serde_json::to_string(
                &az_vm
                    .properties
                    .clone()
                    .unwrap()
                    .hardware_profile
                    .unwrap()
                    .vm_size
                    .unwrap(),
            )
            .expect("Error serializing enum to json string")
            .trim_matches('"')
            .to_string(),
            az: az_vm,
        })
    }
}
pub async fn get_vms(subscription_id: &str) -> Result<VirtualMachines, Box<dyn std::error::Error>> {
    let credential = Arc::new(AzureCliCredential::new());
    let client = azure_mgmt_compute::Client::builder(credential).build()?;
    let mut vms_vec: Vec<VM> = vec![];

    let mut vms_pageable = client
        .virtual_machines_client()
        .list_all(subscription_id)
        .into_stream();
    while let Some(vms) = vms_pageable.next().await {
        let vms = vms?;
        for vm in vms.value {
            // println!("{:?}", &vm.resource.id);
            vms_vec.push(VM::new(vm)?);
        }
    }

    Ok(VirtualMachines { 0: vms_vec })
}
