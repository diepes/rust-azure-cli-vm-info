use azure_core::auth::AccessToken;
use azure_core::{auth::TokenCredential, Url};
// use azure_identity::DefaultAzureCredential;

use serde::Deserialize;

// use std::env;
use std::error::Error;

use azure_identity::AzureCliCredential;
// use futures::stream::StreamExt;
use std::sync::Arc;

//#[tokio::main]
pub async fn az_get_accesstoken() -> Result<AccessToken, Box<dyn Error>> {
    // let credential = DefaultAzureCredential::default();
    let credential = Arc::new(AzureCliCredential::new());
    let response = credential
        .get_token(&["https://management.azure.com/.default"])
        .await?;
    // println!("{:?}", response);
    Ok(response)
}
pub async fn az_get_credentials() -> Result<Arc<AzureCliCredential>, Box<dyn Error>> {
    // let credential = DefaultAzureCredential::default();
    let credentials = Arc::new(AzureCliCredential::new());
    Ok(credentials)
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Subscriptions {
    pub value: Vec<Subscription>,
    pub count: Count,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Count {
    #[serde(rename = "type")]
    count_type: String,
    pub value: usize,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub authorization_source: String,
    pub managed_by_tenants: Vec<Option<serde_json::Value>>,
    pub subscription_id: String,
    pub tenant_id: String,
    pub display_name: String,
    pub state: String,
    pub subscription_policies: SubscriptionPolicies,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPolicies {
    pub location_placement_id: String,
    pub quota_id: String,
    pub spending_limit: String,
}
impl Subscriptions {
    // Function to create a new Vm instance from a JSON string
    pub fn from_json(json_str: &str) -> Result<Subscriptions, serde_json::Error> {
        let subscriptions = serde_json::from_str(json_str);
        subscriptions
    }
    pub fn len(&self) -> usize {
        let count = self.value.len();
        assert_eq!(count, self.count.value);
        count
    }
}

pub async fn az_get_subscriptions(az_token: &AccessToken) -> Result<Subscriptions, Box<dyn Error>> {
    let url = Url::parse(&format!(
        "https://management.azure.com/subscriptions?api-version=2022-12-01"
    ))?;
    let response = reqwest::Client::new()
        .get(url)
        //.header("Authorization", format!("Bearer {}", response.token.secret()))
        .header(
            "Authorization",
            format!("Bearer {}", az_token.token.secret()),
        )
        .send()
        .await?
        .text()
        .await?;

    log::debug!("{:?}", response);
    let subs_json = Subscriptions::from_json(&response).expect("JSON was not well-formatted");
    Ok(subs_json)
}

async fn az_storage(
    az_token: &AccessToken,
    subscription_id: &str,
) -> Result<String, Box<dyn Error>> {
    // NOTE: AZ Tenant(org) has multiple Subscriptions
    // let subscription_id =
    //     env::var("AZURE_SUBSCRIPTION_ID").expect("No 'AZURE_SUBSCRIPTION_ID' env var.");
    let url = Url::parse(&format!(
        "https://management.azure.com/subscriptions/{}/providers/Microsoft.Storage/storageAccounts?api-version=2019-06-01",
        subscription_id))?;
    let response = reqwest::Client::new()
        .get(url)
        //.header("Authorization", format!("Bearer {}", response.token.secret()))
        .header(
            "Authorization",
            format!("Bearer {}", az_token.token.secret()),
        )
        .send()
        .await?
        .text()
        .await?;

    println!("{:?}", response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    //#[test]
    #[tokio::test]
    async fn test_az_get_accesstoken_and_subscriptions() {
        let r = az_get_accesstoken().await;
        assert!(matches!(&r, Ok(_)), "Not logged in with az login ? ");
        let r = r.unwrap();

        let subs = az_get_subscriptions(&r).await;
        assert!(matches!(&subs, Ok(_)), "Not logged in with az login ? ");
        let subs = subs.unwrap();
        assert!(subs.count.value > 0);
        // println!("subs {:?}", subs);

        let st = az_storage(&r, &subs.value[0].subscription_id).await;
        let st_info = st.unwrap_or_else(|s| panic!("Unwrap ERR:`{:#?}`", s));
        let st_string = "idont".to_string();
        assert!(matches!(&st_info, st_string), "Not getting storage info");
    }

    #[test]
    fn test_az_parse_subscriptions() {
        let test_response = concat!("{\"value\":",
            "[{\"id\":\"/subscriptions/7b897b3b-178e-4ec6-aeec-6bab809a5ead\",",
            "\"authorizationSource\":\"RoleBased\",\"managedByTenants\":[],",
            "\"subscriptionId\":\"7b897b3b-178e-4ec6-aeec-6bab809a5ead\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWG Shared Services\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"EnterpriseAgreement_2014-09-01\",\"spendingLimit\":\"Off\"}},",
            "{\"id\":\"/subscriptions/cf4b5afa-85ce-4729-8466-de92d6d57087\",\"authorizationSource\":\"RoleBased\",",
            "\"managedByTenants\":[],\"subscriptionId\":\"cf4b5afa-85ce-4729-8466-de92d6d57087\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWG Shared Services - Sandbox\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"MSDNDevTest_2014-09-01\",\"spendingLimit\":\"Off\"}},",
            "{\"id\":\"/subscriptions/3d6c7977-4632-47a4-8118-b9060e3d35f7\",\"authorizationSource\":\"RoleBased\",",
            "\"managedByTenants\":[],\"subscriptionId\":\"3d6c7977-4632-47a4-8118-b9060e3d35f7\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWG DMZ - Sandbox\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"EnterpriseAgreement_2014-09-01\",\"spendingLimit\":\"Off\"}},",
            "{\"id\":\"/subscriptions/ed6ab5f7-5745-43f1-833b-28a7c06dc330\",\"authorizationSource\":\"RoleBased\",",
            "\"managedByTenants\":[{\"tenantId\":\"c986e767-07af-4c9c-b2a1-08446a3c3e71\"}],",
            "\"subscriptionId\":\"ed6ab5f7-5745-43f1-833b-28a7c06dc330\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWG Shared Services - Non-Prod\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"EnterpriseAgreement_2014-09-01\",\"spendingLimit\":\"Off\"}},",
            "{\"id\":\"/subscriptions/d3f49e41-860f-41e3-b7b7-4cd163f65057\",\"authorizationSource\":\"RoleBased\",",
            "\"managedByTenants\":[],\"subscriptionId\":\"d3f49e41-860f-41e3-b7b7-4cd163f65057\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWG DMZ - Non-Prod\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"EnterpriseAgreement_2014-09-01\",\"spendingLimit\":\"Off\"}},",
            "{\"id\":\"/subscriptions/632b4b77-9278-4368-b3ce-97282afa35b7\",\"authorizationSource\":\"RoleBased\",",
            "\"managedByTenants\":[],\"subscriptionId\":\"632b4b77-9278-4368-b3ce-97282afa35b7\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWG DMZ\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"EnterpriseAgreement_2014-09-01\",\"spendingLimit\":\"Off\"}},",
            "{\"id\":\"/subscriptions/1156dded-08fd-4348-bb97-e24158d3ad1a\",\"authorizationSource\":\"RoleBased\",",
            "\"managedByTenants\":[],\"subscriptionId\":\"1156dded-08fd-4348-bb97-e24158d3ad1a\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWL\\\\WSL\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"EnterpriseAgreement_2014-09-01\",\"spendingLimit\":\"Off\"}},",
            "{\"id\":\"/subscriptions/23ca9af9-8e44-434c-86d5-aa67c56e6896\",\"authorizationSource\":\"RoleBased\",",
            "\"managedByTenants\":[],\"subscriptionId\":\"23ca9af9-8e44-434c-86d5-aa67c56e6896\",",
            "\"tenantId\":\"07e47c7e-9339-473b-8138-adb77559c551\",\"displayName\":\"TWL\\\\WSL - Non-Prod\",",
            "\"state\":\"Enabled\",\"subscriptionPolicies\":{\"locationPlacementId\":\"PublicAndAustralia_2014-09-01\",",
            "\"quotaId\":\"EnterpriseAgreement_2014-09-01\",\"spendingLimit\":\"Off\"}}],",
            "\"count\":{\"type\":\"Total\",\"value\":8}}" );
        let subs_json =
            Subscriptions::from_json(&test_response).expect("JSON was not well-formatted");
        assert_eq!(
            subs_json.count,
            Count {
                count_type: "Total".to_string(),
                value: 8
            }
        );
    }
    #[test]
    fn test_az_parse_simple_subscription() {
        // https://quicktype.io/
        let test_response = r#"
        { "value":[{"id":"/subscriptions/123-456","authorizationSource":"RoleBased","managedByTenants":[],
        "subscriptionId":"123-456","tenantId":"001-002-003","displayName":"TWG Shared 123",
        "state":"Enabled","subscriptionPolicies":{"locationPlacementId":"PublicAndAustralia_2014-09-01",
        "quotaId":"EnterpriseAgreement_2014-09-01","spendingLimit":"Off"}}],
        "count":{"type":"Total","value":18}
        }"#;
        let subs_json =
            Subscriptions::from_json(&test_response).expect("JSON was not well-formatted");
        assert_eq!(
            subs_json.count,
            Count {
                count_type: "Total".to_string(),
                value: 18
            }
        );
    }
    // #[test]
    // fn test_az4() {
    //     let input3 = "curl \"https://mysite.com?\\$filter=name eq 'john' and surname eq 'smith'\"";
    // }
}
