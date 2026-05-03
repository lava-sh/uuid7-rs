fn main() {
    pyo3_build_config::use_pyo3_cfgs();

    // On Windows, pyo3-ffi switched to `raw-dylib` linking and no longer ships
    // an import library for libpython. Mirror the same lib for the whole
    // crate so any local `unsafe extern "C"` block (e.g. PyLongWriter_*) is
    // resolved against the right Python DLL.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && let Some(lib_name) = pyo3_build_config::get().lib_name.as_deref()
    {
        println!("cargo:rustc-link-lib=raw-dylib={lib_name}");
    }
}