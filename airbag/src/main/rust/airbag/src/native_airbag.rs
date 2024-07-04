use std::collections::{HashMap, HashSet};
use std::ptr::null_mut;
use std::sync::Mutex;
use jni::JNIEnv;
use lazy_static::lazy_static;
use libc::{calloc, raise, SA_ONSTACK, SA_RESTART, SA_SIGINFO, sigaction, sigaltstack, sigemptyset, stack_t};
use log::{error, info, warn};
use crate::unwind_utils::get_stack_trace_when_crash;

const SIGNAL_CRASH_STACK_SIZE: usize = 1024 * 128;

#[derive(Clone)]
pub struct NativeAirBagConfig {
    pub(crate) signal_configs: HashMap<i32, HashMap<String, HashSet<String>>>,
}

#[allow(non_snake_case)]
pub fn setConfig(signalConfig: HashMap<i32, HashMap<String, HashSet<String>>>) -> NativeAirBagConfig {
    NativeAirBagConfig { signal_configs: signalConfig }
}

lazy_static! {
    static ref AIR_BAG_CONFIG: Mutex<Option<NativeAirBagConfig>> = Mutex::new(None);
    static ref ORIGINAL_SIGACTIONS: Mutex<Vec<(i32, sigaction)>> = Mutex::new(Vec::new());
}

extern "C" fn sig_handler(sig: i32, _info: *mut libc::siginfo_t, _ptr: *mut libc::c_void) {
    let original_actions = ORIGINAL_SIGACTIONS.lock().unwrap().clone();
    if let Some(ref config) = *AIR_BAG_CONFIG.lock().unwrap() {
        if let Some(signal_config) = config.signal_configs.get(&sig) {
            let crash_trace = get_stack_trace_when_crash();
            info!("dump native stack...");
            info!("{}", crash_trace);
            // Here you would add logic to check the backtrace for keywords
            for elf_name in signal_config.keys() {
                // filter 1. elf_name
                if !elf_name.is_empty() {
                    if crash_trace.contains(elf_name) {
                        match signal_config.get(elf_name) {
                            Some(value) => {
                                // filter 2. keywords.
                                if value.is_empty() {
                                    // if keyword is empty, catch signal directly.
                                    warn!("Caught signal {} from ELF {} with no keyword...", sig, elf_name);
                                    return;
                                }
                                for keyword in value {
                                    if crash_trace.contains(keyword) {
                                        // hit keyword, catch...
                                        warn!("Caught signal {} from ELF {} with keyword {}...", sig, elf_name, keyword);
                                        return;
                                    }
                                }
                            }
                            None => {
                                // ignore
                            }
                        }
                    }
                }
            }
        }

        error!("Signal {} not caught, delegating to original handler...", sig);
        for (orig_sig, orig_action) in original_actions {
            if orig_sig == sig {
                unsafe { sigaction(sig, &orig_action, null_mut()); }
                unsafe { raise(sig); }
            }
        }
    }
}

unsafe fn setup_signal_stack() -> Result<(), &'static str> {
    let ss_sp = calloc(1, SIGNAL_CRASH_STACK_SIZE);
    if ss_sp.is_null() {
        error!("Failed to allocate stack memory");
        return Err("Failed to allocate stack memory");
    }
    let ss = stack_t {
        ss_sp,
        ss_size: SIGNAL_CRASH_STACK_SIZE,
        ss_flags: 0,
    };
    if sigaltstack(&ss, null_mut()) != 0 {
        error!("Failed to set alternate stack");
        return Err("Failed to set alternate stack");
    }
    Ok(())
}

unsafe fn setup_sigaction(mask_signals: &[i32]) -> Result<(), &'static str> {
    let mut sigc: sigaction = std::mem::zeroed();
    sigc.sa_sigaction = sig_handler as usize;
    sigemptyset(&mut sigc.sa_mask);
    sigc.sa_flags = SA_SIGINFO | SA_ONSTACK | SA_RESTART;

    let mut original_sigactions_guard = ORIGINAL_SIGACTIONS.lock().unwrap();
    for &mask_signal in mask_signals {
        let mut original_sigaction: sigaction = std::mem::zeroed();
        if sigaction(mask_signal, &sigc, &mut original_sigaction) == -1 {
            error!("Failed to set signal handler, the signal is {}", mask_signal);
            return Err("Failed to set signal handler");
        }
        original_sigactions_guard.push((mask_signal, original_sigaction));
    }
    Ok(())
}

#[allow(non_snake_case)]
pub unsafe fn register_native_airbag(_env: JNIEnv, nativeAirBagConfig: NativeAirBagConfig) {
    let all_signals: Vec<i32> = nativeAirBagConfig.signal_configs.keys().cloned().collect();

    {
        let mut config_guard = AIR_BAG_CONFIG.lock().unwrap();
        *config_guard = Some(nativeAirBagConfig);
    }

    let setup_result = setup_signal_stack().and_then(|_| setup_sigaction(&all_signals));
    if setup_result.is_ok() {
        info!("Successfully set up signal handlers");
    } else {
        error!("Failed to set up signal handlers");
    }
}