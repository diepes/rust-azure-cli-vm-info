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
pub struct Subscriptions {
    pub value: Vec<azure_mgmt_subscription::models::Subscription>,
}

#[derive(Debug, PartialEq)]
pub struct Subscriptionx {
    pub id: String,
    pub authorization_source: String,
    pub subscription_id: String,
    pub tenant_id: String,
    pub display_name: String,
    pub state: String,
}

impl Subscriptions {
    // Function to create a new Vm instance from a JSON string
    pub async fn new() -> Subscriptions {
        let mut subs_vec: Vec<azure_mgmt_subscription::models::Subscription> = vec![];
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
                subs_vec.push(sub);
            }
        }
        Subscriptions { value: subs_vec }
    }
    pub fn len(&self) -> usize {
        let count = self.value.len();
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
