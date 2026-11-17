use vergen::{Build, Cargo, Emitter, Rustc, Sysinfo};

fn main() {
    let build = Build::builder().build_timestamp(true).build();
    let cargo = Cargo::builder()
        .features(true)
        .debug(true)
        .opt_level(true)
        .target_triple(true)
        .dependencies(true)
        .build();
    let rustc = Rustc::builder().semver(true).channel(true).build();
    let si = Sysinfo::builder().os_version(true).build();

    Emitter::default()
        .add_instructions(&build)
        .unwrap()
        .add_instructions(&cargo)
        .unwrap()
        .add_instructions(&rustc)
        .unwrap()
        .add_instructions(&si)
        .unwrap()
        .quiet()
        .emit()
        .unwrap();
}
