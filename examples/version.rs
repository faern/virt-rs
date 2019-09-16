fn main() {
    match virt::version::version() {
        Ok(v) => println!(
            "Using libvirt version {}.{}.{}",
            v.major, v.minor, v.release
        ),
        Err(e) => eprintln!("Unable to fetch version: {}", e),
    }
}
