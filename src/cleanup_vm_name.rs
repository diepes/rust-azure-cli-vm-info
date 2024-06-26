#[derive(Debug, PartialEq)]
pub struct VmNameAndPrice {
    pub name: String,
    pub price_type: VmPriceType,
}

#[derive(Debug, PartialEq)]
pub enum VmPriceType {
    Spot,
    SpotWindows,
    LowPriority,
    LowPriorityWindows,
    DevTest,
    DevTestWindows,
    Normal,
    NormalWindows,
}
impl VmPriceType {
    pub fn to_str(&self) -> &str {
        match self {
            VmPriceType::Spot => "Spot",
            VmPriceType::SpotWindows => "Spot Windows",
            VmPriceType::LowPriority => "Low Priority",
            VmPriceType::LowPriorityWindows => "Low Priority Windows",
            VmPriceType::DevTest => "DevTest",
            VmPriceType::DevTestWindows => "DevTestWindows",
            VmPriceType::Normal => "",
            VmPriceType::NormalWindows => "Windows",
        }
    }
}

pub fn split_price_type(
    price_option: &crate::az::pricing::retrieve::Pricing,
) -> (String, VmPriceType) {
    let price_type: VmPriceType;
    let vm_name: String;
    let name_in = &price_option.sku_name;
    let flag_windows = price_option.product_name.ends_with("Windows");
    if price_option.type_bill == "DevTestConsumption" {
        price_type = if flag_windows {
            VmPriceType::DevTestWindows
        } else {
            VmPriceType::DevTest
        };
        vm_name = name_in.trim().to_string();
    } else if let Some(name) = name_in.strip_suffix(VmPriceType::Spot.to_str()) {
        price_type = if flag_windows {
            VmPriceType::SpotWindows
        } else {
            VmPriceType::Spot
        };
        vm_name = name.trim().to_string();
    } else if let Some(name) = name_in.strip_suffix(VmPriceType::LowPriority.to_str()) {
        price_type = if flag_windows {
            VmPriceType::LowPriorityWindows
        } else {
            VmPriceType::LowPriority
        };
        vm_name = name.trim().to_string();
    } else {
        price_type = if flag_windows {
            VmPriceType::NormalWindows
        } else {
            VmPriceType::Normal
        };
        vm_name = name_in.to_string();
    }
    (vm_name, price_type)
}

pub fn split_and_cleanup_name(name_in: &str) -> Vec<String> {
    // let sn = name_in.strip_prefix("Standard_").unwrap_or(name_in);
    //let sn = sn.replace("_", " ");
    name_in
        .split("/")
        .map(|s| s.strip_prefix("Standard_").unwrap_or(s).replace("_", " "))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    //#[test]
    #[tokio::test]
    async fn test_split_price_type() {
        let price_option = crate::az::pricing::retrieve::Pricing {
            sku_name: "Standard_DS11_v2".to_string(),
            product_name: "Standard_DS11_v2".to_string(),
            type_bill: "DevTestConsumption".to_string(),
            ..Default::default()
        };
        assert_eq!(
            split_price_type(&price_option),
            ("Standard_DS11_v2".to_string(), VmPriceType::DevTest)
        );
        // assert_eq!(
        //     split_price_type(&price_option),
        //     ("MyVm/3ndVm".to_string(), VmPriceType::Normal)
        // );
        // assert_eq!(
        //     split_price_type(&price_option),
        //     ("MyVm/4ndVm".to_string(), VmPriceType::DevTest)
        // );
        // assert_eq!(
        //     split_price_type(&price_option),
        //     ("MyVm/4ndVm".to_string(), VmPriceType::LowPriority)
        // );
    }

    #[test]
    fn test_split_and_cleanup_name() {
        assert_eq!(split_and_cleanup_name("MyVm/2ndVm"), vec!["MyVm", "2ndVm"]);
        assert_eq!(
            split_and_cleanup_name("Standard_MyVm/3ndVm"),
            vec!["MyVm", "3ndVm"]
        );
        assert_eq!(
            split_and_cleanup_name("Standard_MyVm/Standard_4thVm"),
            vec!["MyVm", "4thVm"]
        );
        assert_eq!(
            split_and_cleanup_name("Standard_DS11-1_v2"),
            vec!["DS11-1 v2"]
        );
        assert_eq!(split_and_cleanup_name("Standard_DS11_v2"), vec!["DS11 v2"]);
    }
    #[test]
    fn test_az_parse_simple_subscription() {}
    // #[test]
    // fn test_az4() {
    //     let input3 = "curl \"https://mysite.com?\\$filter=name eq 'john' and surname eq 'smith'\"";
    // }
}
