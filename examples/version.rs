use virt::connection::Connection;

fn main() {
    match virt::version::lib_version() {
        Ok(v) => println!(
            "Using libvirt version {}.{}.{}",
            v.major, v.minor, v.release
        ),
        Err(e) => eprintln!("Unable to get local lib version: {}", e),
    }

    let connection = Connection::open_default().expect("Unable to connect");
    println!("Hypervisor:");
    println!("\tURI: {}", connection.uri().expect("Unable to get URI"));
    println!("\thostname: {}", connection.hostname().unwrap());
    println!(
        "\thypervisor type: {}",
        connection.hypervisor_type().unwrap()
    );
    println!(
        "\thypervisor version: {}",
        connection
            .hypervisor_version()
            .unwrap()
            .map(|v| v.to_string())
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Unknown")
    );
    println!("\tlibvirt version: {}", connection.lib_version().unwrap());
}
