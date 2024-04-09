/*
USE the AZURE auto gen code

cargo add azure_mgmt_compute
cargo add azure_mgmt_subscription
*/

use azure_identity::AzureCliCredential;
use futures::stream::StreamExt;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct VirtualMachines( pub Vec<VM>);

#[derive(Debug, PartialEq)]
pub struct VM {
    pub id: String,
    pub name: String,
    pub location: String,
    pub az: azure_mgmt_compute::models::VirtualMachine, // Value from Rust Azure generated api
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
            vms_vec.push(VM {
                id: vm.resource.id.clone().unwrap(),
                name: vm.resource.name.clone().unwrap(),
                location: vm.resource.location.clone(),
                az: vm,
            });
        }
    }

    Ok(VirtualMachines { 0: vms_vec })
}
