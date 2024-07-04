use std::ffi::CStr;
use std::fmt::Write;
use std::ptr;

use libc::{c_char, c_void, uintptr_t};

extern "C" {
    fn _Unwind_Backtrace(callback: extern "C" fn(ctx: *mut _Unwind_Context, arg: *mut c_void) -> _Unwind_Reason_Code, arg: *mut c_void) -> _Unwind_Reason_Code;
    fn _Unwind_GetIP(ctx: *mut _Unwind_Context) -> uintptr_t;
    fn dladdr(addr: *const c_void, info: *mut Dl_info) -> i32;
}

#[repr(C)]
struct _Unwind_Context;

#[repr(C)]
enum _Unwind_Reason_Code {
    _URC_NO_REASON = 0,
    _URC_END_OF_STACK = 5,
}

#[repr(C)]
struct Dl_info {
    dli_fname: *const c_char,
    dli_fbase: *const c_void,
    dli_sname: *const c_char,
    dli_saddr: *const c_void,
}

struct BacktraceState {
    current: *mut *mut c_void,
    end: *mut *mut c_void,
}

extern "C" fn unwind_callback(ctx: *mut _Unwind_Context, arg: *mut c_void) -> _Unwind_Reason_Code {
    let state = unsafe { &mut *(arg as *mut BacktraceState) };
    let pc = unsafe { _Unwind_GetIP(ctx) } as *mut c_void;
    if !pc.is_null() {
        if state.current == state.end {
            return _Unwind_Reason_Code::_URC_END_OF_STACK;
        } else {
            unsafe {
                *state.current = pc;
                state.current = state.current.add(1);
            }
        }
    }
    _Unwind_Reason_Code::_URC_NO_REASON
}

fn capture_backtrace(buffer: &mut [*mut c_void]) -> usize {
    unsafe {
        let mut state = BacktraceState {
            current: buffer.as_mut_ptr(),
            end: buffer.as_mut_ptr().add(buffer.len()),
        };
        _Unwind_Backtrace(unwind_callback, &mut state as *mut _ as *mut c_void);
        state.current.offset_from(buffer.as_mut_ptr()) as usize
    }
}

fn dump_backtrace(buffer: &[*mut c_void], count: usize) -> String {
    unsafe {
        let mut output = String::new();
        for (idx, &addr) in buffer.iter().take(count).enumerate() {
            let mut info: Dl_info = std::mem::zeroed();
            let symbol = if dladdr(addr as *const c_void, &mut info) != 0 && !info.dli_sname.is_null() {
                CStr::from_ptr(info.dli_sname).to_str().unwrap_or("")
            } else {
                ""
            };
            let filename = if !info.dli_fname.is_null() {
                CStr::from_ptr(info.dli_fname).to_str().unwrap_or("")
            } else {
                ""
            };
            let _ = write!(
                output,
                "  #{} at {}: {:p}  {}\n",
                idx,
                filename,
                addr,
                symbol
            );
        }
        output
    }
}

pub fn get_stack_trace_when_crash() -> String {
    const MAX: usize = 30;
    let mut buffer: [*mut c_void; MAX] = [ptr::null_mut(); MAX];
    let count = capture_backtrace(&mut buffer);
    dump_backtrace(&buffer, count)
}