use azure_vm_info::{self, az}; // Import lib.rs (library)
use log4rs;
//use tokio::main;
use std::error::Error;

// use azure_identity::AzureCliCredential;
// use futures::stream::StreamExt;

use azure_identity::AzureCliCredential;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Do as little as possible in main.rs as it can't contain any tests
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    dotenv::dotenv().ok();
    //
    log::info!("#Start main()");
    let access_token = azure_vm_info::az::identity::az_get_accesstoken().await?;
    log::info!("got az accessToken {:?}", access_token);
    //
    let credential = Arc::new(AzureCliCredential::new());
    //
    // // Subscriptions 1/2
    // let subs = azure_vm_info::az::identity::az_get_subscriptions(&access_token).await?;
    // log::info!("got az subscriptions 1/2: #{}", subs.len());
    // Subscriptions 2/2 - Uses the generated Azure Rust
    let subs = az::subscriptions::Subscriptions::new().await;
    log::debug!("subscriptions 2/2: {:?}", subs);
    log::info!("got az subscriptions 2/2: #{}", subs.len());

    //
    // let mut vms = azure_vm_info::az::vmlist::az_get_vmlist(credential, subs)?;
    // let mut vms = az_vms::get_fake()?;
    // log::info!("DEBUG subscription = {:?}", subs.value[0]);
    for subscription in &subs.value {
        let subscription_id = &(subscription.subscription_id.clone().unwrap());
        // log::info!("get VMs for subscription = {:?}", subscription);
        let vms = az::vmlist::get_vms(subscription_id).await;
        log::info!("vms: #{:?} subscription: {}", vms.unwrap().value.len(), subscription_id);
    }

    //log::info!("got all vm's {}", vms.len());

    log::debug!("add flex group and ratios for each vm");
    //azure_vm_info::enrich_vm_fields(&mut vms);

    //# print_vms(&vms, &print_keys, &az_sub);
    //azure_vm_info::print_summary(&vms)?;
    Ok(())
}
pub fn run() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    log::info!("#Start run()");

    //let mut vms = az_vms::get_all()?;
    //let mut vms = az_vms::get_fake()?;

    //log::info!("got all vm's {}", vms.len());

    log::debug!("add flex group and ratios for each vm");
    // enrich_vm_fields(&mut vms);

    //# print_vms(&vms, &print_keys, &az_sub);
    // print_summary(&vms)?;
    Ok(())
}
