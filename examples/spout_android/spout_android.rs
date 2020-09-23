use ndk::trace;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    let _trace;
    if trace::is_trace_enabled() {
        _trace = trace::Section::new("ndk-rs example main").unwrap();
    }

    spout::spout_main::run();
}
