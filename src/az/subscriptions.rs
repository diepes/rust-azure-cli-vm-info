/*
USE the AZURE auto gen code

cargo add azure_mgmt_compute
cargo add azure_mgmt_subscription
*/

use azure_identity::AzureCliCredential;
use futures::stream::StreamExt;
use std::sync::Arc;
//#[tokio::main]

#[derive(Debug, PartialEq)]
pub struct Subscriptions(pub Vec<Subscription>);

#[derive(Debug, PartialEq)]
pub struct Subscription {
    pub subscription_id: String,
    pub display_name: String,
    pub state: azure_mgmt_subscription::models::subscription::State,
    pub az: azure_mgmt_subscription::models::Subscription, // Value from Rust Azure generated api
}

impl Subscriptions {
    // Function to create a new Vm instance from a JSON string
    pub async fn new() -> Subscriptions {
        let mut subs_vec: Vec<Subscription> = vec![];
        let credential = Arc::new(AzureCliCredential::new());
        let subscriptions_client = azure_mgmt_subscription::Client::builder(credential)
            .build()
            .expect("Err with az login ?");
        let mut subscriptions_pagable = subscriptions_client
            .subscriptions_client()
            .list()
            .into_stream();
        while let Some(subscriptions) = subscriptions_pagable.next().await {
            for sub in subscriptions.unwrap().value {
                // println!("sub: {:?}\n\n", sub);
                subs_vec.push(Subscription {
                    subscription_id: sub.subscription_id.clone().unwrap(),
                    display_name: sub.display_name.clone().unwrap(),
                    state: sub.state.clone().unwrap(),
                    az: sub,
                });
            }
        }
        Subscriptions{ 0: subs_vec }
    }
    pub fn len(&self) -> usize {
        let count = self.0.len();
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //#[test]
    #[tokio::test]
    async fn test_az_get_accesstoken_and_subscriptions() {}
}
