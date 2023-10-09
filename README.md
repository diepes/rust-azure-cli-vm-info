# rust-azure-cli-vm-info

⛔️ Broken ⛔️

Retrieve Azure VM info by running az-cli in rust
Requires a working azure-cli, as it runs az commands.

## Flex group

* Azure flex group, allows for reservaton of small size vm in a family group and then apply the savings to it and bigger vms.
* See - ./isfrationblob.csv we got from [https://learn.microsoft.com/en-us/azure/virtual-machines/reserved-vm-instance-size-flexibility]
  * [https://aka.ms/isf]

## Code overview

provides a ```cmd::run``` function to wrap ```std::process::Command```

* e.g.

      my_obj = cmd::run("az account list --query '[].name' --output json");

* Allows string as commandline, no need to split into vec.
* parses the string, split on space, but respects single quotes.
* expects json output, and parses it.
