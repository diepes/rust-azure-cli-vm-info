use azure_vm_info::{self, az}; // Import lib.rs (library)
use log4rs;
//use tokio::main;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Do as little as possible in main.rs as it can't contain any tests
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    dotenv::dotenv().ok();
    //
    log::info!("#Start main()");

    let subs = az::subscriptions::Subscriptions::new().await;
    let count_total_subscriptions = subs.len();
    log::debug!("subscriptions 2/2: {:?}", subs);
    log::info!("got az subscriptions 2/2: #{}", count_total_subscriptions);

    let mut count_total_vms = 0;
    for subscription in &subs.value {
        let subscription_id = &(subscription.subscription_id.clone().unwrap());
        let vms = az::vmlist::get_vms(subscription_id).await;
        let cnt_vms = vms.unwrap().value.len();
        count_total_vms += cnt_vms;
        log::info!("vms: #{:?} subscription: {}", cnt_vms, subscription_id);
    }
    log::info!(
        "Total vms: #{} subscription: #{}",
        count_total_vms,
        count_total_subscriptions
    );

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
