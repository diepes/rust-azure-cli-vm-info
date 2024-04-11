// use azure_mgmt_compute::models::VirtualMachine;
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
    let mut vms = az::vmlist::VirtualMachines { 0: vec![] };
    for subscription in &subs.0 {
        let subscription_id = &subscription.subscription_id;
        let subscription_name = &subscription.display_name;
        let mut vms_sub = az::vmlist::get_vms(subscription_id)
            .await
            .expect("Error retrieving VM's");
        let cnt_vms_sub = vms_sub.0.len();
        // for vm in vms.value {
        //     println!("VM: {} id:{}", vm.name, vm.id);
        // }
        count_total_vms += cnt_vms_sub;
        vms.0.append(&mut vms_sub.0);
        log::info!(
            "vms: #{:2} subscription: '{}' {}",
            cnt_vms_sub,
            subscription_name,
            subscription_id
        );
    }
    log::info!(
        "Total vms: #{}={} subscription: #{}",
        count_total_vms,
        vms.0.len(),
        count_total_subscriptions
    );

    log::debug!("add flex group and ratios for each vm");
    azure_vm_info::enrich_vm_fields(&mut vms);

    //azure_vm_info::print_vms(&vms, &print_keys, &az_sub);
    azure_vm_info::print_summary(&vms)?;
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
