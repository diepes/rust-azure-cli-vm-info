/// Define keys we want from Azure VM's
///
use serde::Deserialize;

// #[derive(Debug, Deserialize)]
// pub struct Vm {
//     // "additionalCapabilities", ""),
//     // "applicationProfile", ""),
//     // "availabilitySet", ""),
//     #[serde(rename = "billingProfile.maxPrice")]
//     pub billing_profile: Option<String>,
//     // "capacityReservation", ""),
//     // "diagnosticsProfile", ""),
//     // "evictionPolicy", ""),
//     // "extendedLocation", ""),
//     // "extensionsTimeBudget", ""),
//     pub fqdns: String, // --show-details
//     #[serde(rename = "hardwareProfile")]
//     pub hardware_profile: HardwareProfile,
//     // "host", ""),
//     // "hostGroup", ""),
//     // "id", ""),
//     // "identity", ""),
//     // "instanceView", ""),
//     // "licenseType", ""),
//     // "location", ""),
//     // "macAddresses", ""), // --show-details
//     pub name: String,
//     // "networkProfile", ""),
//     // "osProfile", ""),
//     // "plan", ""),
//     // "platformFaultDomain", ""),
//     #[serde(rename = "powerState")]
//     pub power_state: String, // --show-details
//     // "priority", ""),
//     #[serde(rename = "privateIps")]
//     pub private_ips: String, // --show-details
//     // "provisioningState", ""),
//     // "proximityPlacementGroup", ""),
//     #[serde(rename = "publicIps")]
//     pub public_ips: String, // --show-details
//     #[serde(rename = "resourceGroup")]
//     pub resource_group: String,
//     // "resources", ""),
//     // "scheduledEventsProfile", ""),
//     // "securityProfile", ""),
//     // "storageProfile", ""),
//     pub tags: Option<std::collections::BTreeMap<String, String>>,
//     // "timeCreated", ""),
//     // "type", ""),
//     // "userData", ""),
//     // "virtualMachineScaleSet", ""),
//     #[serde(rename = "vmId")]
//     pub vm_id: String,
//     pub zones: Option<Vec<String>>,
//     pub flex_lookup: Option<FlexLookUp>, // Added from csv lookup of flex options.
// }

// #[derive(Debug, Deserialize)]
// pub struct HardwareProfile {
//     #[serde(rename = "vmSize")]
//     pub vm_size: String,
// }

// needs Clone so we can make a copy to read Some().fields
#[derive(Debug, Deserialize, Clone)]
pub struct FlexLookUp {
    pub flex_group: String,
    pub flex_sku_name: String,
    pub flex_ratio: String,
    pub flex_options: String,
}

// impl Vm {
//     // Function to create a new Vm instance from a JSON string
//     pub fn from_json(json_str: &str) -> Result<Vm, serde_json::Error> {
//         serde_json::from_str(json_str)
//     }
//     // Function to create a Vec<Vm> from a JSON array string
//     pub fn from_json_array(json_str: &str) -> Result<Vec<Vm>, serde_json::Error> {
//         serde_json::from_str(json_str)
//     }
// }
