extern crate android_logger;
extern crate log;

use std::sync::{Arc, Mutex};

use android_logger::Config;
use jni::JavaVM;
use jni::sys::{jint, JNI_VERSION_1_6};
use lazy_static::lazy_static;
use log::{info, LevelFilter};

lazy_static! {
    static ref GLOBAL_JVM: Arc<Mutex<Option<JavaVM>>> = Arc::new(Mutex::new(None));
}

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, _: *mut std::ffi::c_void) -> jint {
    
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    info!("native_airbag has been initialized");
    
    let java_vm = unsafe { JavaVM::from_raw(vm).expect("Failed to create JavaVM from raw pointer") };
    {
        let mut global_vm = GLOBAL_JVM.lock().unwrap();
        *global_vm = Some(java_vm);
    }

    JNI_VERSION_1_6
}

#[no_mangle]
pub extern "system" fn JNI_OnUnload(_: *mut jni::sys::JavaVM, _: *mut std::ffi::c_void) {
    // 清理全局变量
    let mut global_vm = GLOBAL_JVM.lock().unwrap();
    *global_vm = None;
}