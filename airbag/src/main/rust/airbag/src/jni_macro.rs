// jni_macro.rs
#[macro_export]
macro_rules! jstring_to_string {
    ($env:expr, $java_string:expr) => {{
        let jstring_obj = unsafe { JString::from_raw($java_string) };
        let c_str = $env.get_string(&jstring_obj).expect("Couldn't get java string!");
        let c_str: &std::ffi::CStr = c_str.as_ref();
        let rust_string = c_str.to_str().expect("Couldn't convert CStr to &str!").to_owned();
        rust_string
    }};
}