use pyo3_build_config::PythonImplementation;

fn main() {
    pyo3_build_config::use_pyo3_cfgs();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let config = pyo3_build_config::get();

    if let Some(lib_name) = config.lib_name().as_ref() {
        println!("cargo:rustc-cfg=pyo3_dll=\"{lib_name}\"");
    } else if config.implementation() == PythonImplementation::CPython && !config.is_free_threaded()
    {
        println!(
            "cargo:rustc-cfg=pyo3_dll=\"python{}{}\"",
            config.version().major,
            config.version().minor
        );
    }
}
