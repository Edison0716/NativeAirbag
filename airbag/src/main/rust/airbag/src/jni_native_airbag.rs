use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use log::info;
use crate::jstring_to_string;
use crate::native_airbag::{register_native_airbag, setConfig};

use std::collections::{HashMap, HashSet};

// {
//     "signalConfig": {
//         "11": {
//             "edsion1.so": ["A", "B"],
//             "edsion2.so": ["A"],
//             "test.so": ["AA", "BB"]
//         },
//     }
// }
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_edisonlivings_airbag_Airbag_registerNativeAirbag(
    mut env: JNIEnv,
    _: JClass,
    jsonConfig: jstring
) {
    let jsonConfig: String = jstring_to_string!(&mut env, jsonConfig);
    info!("masked elf is {}", jsonConfig);
    let signalConfig: HashMap<i32, HashMap<String, HashSet<String>>> = serde_json::from_str(&*jsonConfig).expect("JSON was not well-formatted");
    unsafe {
        register_native_airbag(env, setConfig(signalConfig));
    }
}


#[no_mangle]
pub extern "C" fn Java_com_edisonlivings_airbag_Airbag_sendSignal(_env: JNIEnv, _: JClass, signal: jint) {
    info!("receive send signal {}", signal);
    unsafe { libc::raise(signal); }
}


