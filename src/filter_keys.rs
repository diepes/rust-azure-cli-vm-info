/// Define keys we want from Azure VM's
///
fn get_key_map() -> Vec<(usize, &'static str, &'static str)> {
    // Keys witn 0 - removed, highter num sorted by priority 1-> 99
    vec![
        (0, "additionalCapabilities", ""),
        (0, "applicationProfile", ""),
        (0, "availabilitySet", ""),
        (99, "billingProfile", ""),
        (0, "capacityReservation", ""),
        (0, "diagnosticsProfile", ""),
        (0, "evictionPolicy", ""),
        (0, "extendedLocation", ""),
        (0, "extensionsTimeBudget", ""),
        (99, "fqdns", ""), // --show-details
        (14, "hardwareProfile", "vmSize"),
        (0, "host", ""),
        (0, "hostGroup", ""),
        (0, "id", ""),
        (0, "identity", ""),
        (0, "instanceView", ""),
        (0, "licenseType", ""),
        (0, "location", ""),
        (0, "macAddresses", ""), // --show-details
        (1, "name", ""),
        (0, "networkProfile", ""),
        (0, "osProfile", ""),
        (0, "plan", ""),
        (0, "platformFaultDomain", ""),
        (12, "powerState", ""), // --show-details
        (0, "priority", ""),
        (99, "privateIps", ""), // --show-details
        (0, "provisioningState", ""),
        (0, "proximityPlacementGroup", ""),
        (99, "publicIps", ""), // --show-details
        (15, "resourceGroup", ""),
        (0, "resources", ""),
        (0, "scheduledEventsProfile", ""),
        (0, "securityProfile", ""),
        (0, "storageProfile", ""),
        (14, "tags", ""),
        (0, "timeCreated", ""),
        (0, "type", ""),
        (0, "userData", ""),
        (0, "virtualMachineScaleSet", ""),
        (20, "vmId", ""),
        (16, "zones", ""),
    ]
}
pub fn get_key_sort_order() -> Vec<(String, String)> {
    let mut keys_in: Vec<(usize, &'static str, &'static str)> = get_key_map();
    keys_in.sort_by(|a, b| a.0.cmp(&b.0));

    let keys_out: Vec<(String, String)> = keys_in
        .iter()
        .filter(|&&(a, _, _)| a > 0) // return = true
        .map(|v| (v.1.to_string(), v.2.to_string()))
        .collect::<Vec<(String, String)>>();

    keys_out
}
pub fn remove_key(k: &str) -> bool {
    // match keys to ignore (return false)
    for (prio, key, _sub_key) in get_key_map() {
        if prio == 0 && key == k {
            return true;
        }
    }
    return false;
}
