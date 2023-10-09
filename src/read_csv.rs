use std::cell::OnceCell;
use std::collections::HashMap;
/// Define keys we want from Azure VM's
///
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug, PartialEq)]
pub struct CsvRow {
    pub flex_group: String,
    pub flex_sku_name: String,
    pub flex_ratio: String,
}
pub fn get_azure_flex_groups_from_csv() -> OnceCell<(Vec<CsvRow>, HashMap<String, Vec<String>>)> {
    // Open the CSV file
    // https://aka.ms/isf
    let file = File::open("isfratioblob.csv").expect("Missing file isfratioblob.csv ???");
    let reader = BufReader::new(file);

    // Read and parse the CSV data
    let mut data = Vec::new();
    let mut data_flex: HashMap<String, Vec<String>> = HashMap::new();
    let mut is_first_line = true;
    for line in reader.lines() {
        let line = line.expect("Error reading line from csv");
        if is_first_line {
            is_first_line = false;
            if line.trim() != "InstanceSizeFlexibilityGroup,ArmSkuName,Ratio" {
                panic!("Invalid header line in inport file");
            }
            continue; // Skip the header line
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 3 {
            panic!("Invalid number of columns");
        }

        let flex_group = fields[0].to_string();
        let flex_sku_name = fields[1].to_string();
        let flex_ratio = fields[2].to_string();
        if flex_ratio == "1" {
            if data_flex.get(&flex_group).is_none() {
                data_flex.insert(flex_group.clone(), vec![flex_sku_name.clone()]);
            } else {
                data_flex
                    .get_mut(&flex_group)
                    .map(|v| v.push(flex_sku_name.clone()));
            }
        }

        let _test_ratio: f64 = flex_ratio
            .parse::<f64>()
            .map_err(|_| "Failed to parse ratio as number")
            .expect(&format!("Failed to parse ratio as number {:?}", fields));

        data.push(CsvRow {
            flex_group,
            flex_sku_name,
            flex_ratio,
        });
    }

    // Create a OnceCell containing the parsed data
    let once_cell = OnceCell::new();
    if once_cell.set((data, data_flex)).is_err() {
        panic!("Data already set in OnceCell");
    }

    once_cell
}
