# rust-azure-cli-vm-info

⛔️ Broken ⛔️

Retrieve Azure VM info by running az-cli in rust
Requires a working azure-cli, as it runs az commands.

## Flex group

* Azure flex group, allows for reservaton of small size vm in a family group and then apply the savings to it and bigger vms.
* See - ./isfrationblob.csv we got from [https://learn.microsoft.com/en-us/azure/virtual-machines/reserved-vm-instance-size-flexibility]
  * [https://aka.ms/isf]
  * 2024-04-10 missing 'Standard_A2_v2' update isfrationblob.csv with latest fInstance size flex ratios.

## Run

1. login ```az login```
2. select correct tenant ```az login --tenant <tennant_id>```
3. cargo run

## Code overview

* Updated away from cmd::run to run az cli, to calling rust Azure api

* Tests cargo test

provides a ```cmd::run``` function to wrap ```std::process::Command```

* e.g.

      my_obj = cmd::run("az account list --query '[].name' --output json");

* Allows string as commandline, no need to split into vec.
* parses the string, split on space, but respects single quotes.
* expects json output, and parses it.
