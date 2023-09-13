# rust-azure-cli-vm-info

Retrieve Azure VM info by running az-cli in rust

Requires a working azure-cli, as it runs az commands.

## Code overview

provides a ```cmd::run``` function to wrap ```std::process::Command```

* e.g.

      my_obj = cmd::run("az account list --query '[].name' --output json");

* Allows string as commandline, no need to split into vec.
* parses the string, split on space, but respects single quotes.
* expects json output, and parses it.
