// Compiles the Swiss Ephemeris C library (vendor/swisseph, AGPL-3.0) into the app.
fn main() {
    let files = [
        "swedate.c", "swehouse.c", "swejpl.c", "swemmoon.c", "swemplan.c", "sweph.c", "swephlib.c", "swecl.c", "swehel.c",
    ];
    let mut build = cc::Build::new();
    for f in files {
        build.file(format!("vendor/swisseph/{f}"));
        println!("cargo:rerun-if-changed=vendor/swisseph/{f}");
    }
    build.include("vendor/swisseph").warnings(false).opt_level(2).compile("swisseph");
}
