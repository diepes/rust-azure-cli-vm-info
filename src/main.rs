// Import lib.rs (library)
use azure_vm_info;
use log4rs;
fn main() {
    // Do as little as possible in main.rs as it can't contain any tests
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    match azure_vm_info::run() {
        Ok(_) => {
            println!("The End.");
        }
        Err(error) => {
            println!("RUN error Err: '{}'", error);
        }
    }
}
