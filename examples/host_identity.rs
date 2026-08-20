//! Documented host-identity output using overrides (no rustc-env required).

fn main() {
    let info = rsfulmen::buildinfo::resolve_with_overrides(
        "0.1.5",
        "a1b2c3d4e5f6a7b8",
        "2026-08-20T00:00:00Z",
        "false",
    );
    println!("{}", info.format_basic("example"));
    println!();
    println!(
        "{}",
        info.format_extended("example", Some(&rsfulmen::buildinfo::Pins::from_crate()))
    );
    println!(
        "{}",
        info.to_json("example", Some(&rsfulmen::buildinfo::Pins::from_crate()))
    );
}
