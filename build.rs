fn main() {
    pyo3_build_config::use_pyo3_cfgs();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && let Some(lib_name) = pyo3_build_config::get().lib_name.as_deref()
    {
        println!("cargo:rustc-cfg=pyo3_dll=\"{lib_name}\"");
    }
}
