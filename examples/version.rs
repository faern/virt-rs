use virt::connection::Connection;

fn main() {
    match virt::version::lib_version() {
        Ok(v) => println!(
            "Using libvirt version {}.{}.{}",
            v.major, v.minor, v.release
        ),
        Err(e) => eprintln!("Unable to get local lib version: {}", e),
    }
    match Connection::open_default() {
        Ok(connection) => match connection.lib_version() {
            Ok(v) => println!(
                "Hypervisor uses libvirt version {}.{}.{}",
                v.major, v.minor, v.release
            ),
            Err(e) => eprintln!("Unable to fetch hypervisor lib version: {}", e),
        },
        Err(e) => eprintln!("Unable to connect to a default hypervisor: {}", e),
    }
}
