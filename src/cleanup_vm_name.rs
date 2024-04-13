#[derive(Debug, PartialEq)]
pub struct VmNameAndPrice {
    pub name: String,
    pub price_type: VmPriceType,
}

#[derive(Debug, PartialEq)]
pub enum VmPriceType {
    Spot,
    LowPriority,
    Normal,
}
impl VmPriceType {
    pub fn to_str(&self) -> &str {
        match self {
            VmPriceType::Spot => "Spot",
            VmPriceType::LowPriority => "Low Priority",
            VmPriceType::Normal => "",
        }
    }
}

pub fn split_price_type(name_in: &str) -> (String, VmPriceType) {
    let price_type: VmPriceType;
    let vm_name: String;
    if let Some(name) = name_in.strip_suffix(VmPriceType::Spot.to_str()) {
        price_type = VmPriceType::Spot;
        vm_name = name.trim().to_string();
    } else if let Some(name) = name_in.strip_suffix(VmPriceType::LowPriority.to_str()) {
        price_type = VmPriceType::LowPriority;
        vm_name = name.trim().to_string();
    } else {
        price_type = VmPriceType::Normal;
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
        assert_eq!(
            split_price_type(&"MyVm/2ndVm Spot"),
            ("MyVm/2ndVm".to_string(), VmPriceType::Spot)
        );
        assert_eq!(
            split_price_type(&"MyVm/3ndVm"),
            ("MyVm/3ndVm".to_string(), VmPriceType::Normal)
        );
        assert_eq!(
            split_price_type(&"MyVm/4ndVm Low Priority"),
            ("MyVm/4ndVm".to_string(), VmPriceType::LowPriority)
        );
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
