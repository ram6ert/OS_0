use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let kernel = PathBuf::from(&args[1]);
    let out_dir = PathBuf::from(&args[2]);

    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .unwrap();
}
