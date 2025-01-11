use std::env;

fn main() {
    println!("cargo::rustc-env=CH_PROFILE={}", env::var("PROFILE").unwrap());
    println!("cargo::rustc-env=CH_HOST={}", env::var("HOST").unwrap());
    println!("cargo::rustc-env=CH_TARGET={}", env::var("TARGET").unwrap());
    println!("cargo::rustc-env=CH_BUILDSCRIPT=rev-1");
}