fn main() {
    println!("cargo:rustc-link-search=native=/usr/lib/digilent/adept");
    println!("cargo:rustc-link-lib=dylib=depp");
    println!("cargo:rustc-link-lib=dylib=djtg");
    println!("cargo:rustc-link-lib=dylib=dmgr");
}
