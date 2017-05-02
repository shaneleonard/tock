use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn kernel_attribute_git() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("kernel_attribute_git.rs");
    let mut f = File::create(&dest_path).unwrap();

    f.write_all(b"
#[link_section=\".kernel_attribute.git\"]
#[no_mangle]
pub static KERNEL_ATTRIBUTE_GIT: [u8; 64] = [
    ").unwrap();

    // "git" padded to 8 bytes
    f.write_all(b"0x67, 0x69, 0x74, 0x0, 0x0, 0x0, 0x0, 0x0, ").unwrap();

    let attr_str = env::var("TOCK_KERNEL_VERSION").unwrap();

    // attribute length
    let _ = write!(f, "{:#x}, ", attr_str.len());

    // copy the attribute itself
    for b in attr_str.bytes() {
        let _ = write!(f, "{:#x}, ", b);
    }

    // pad the rest with 0's
    for _ in attr_str.len() .. 55 {
        let _ = write!(f, "0x0, ");
    }

    // And finish the array
    f.write_all(b" ]; ").unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    kernel_attribute_git();
}
