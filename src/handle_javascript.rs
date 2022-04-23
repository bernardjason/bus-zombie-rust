use std::ffi::{CStr, };
use std::os::raw::c_char;
use std::sync::Mutex;

lazy_static! {
    static ref DATA_STATS: Mutex<String> = Mutex::new(String::with_capacity(4096));
}

pub fn write_stats_data(output: *const c_char) {
    unsafe {
        let rust = CStr::from_ptr(output);
        let mut data = DATA_STATS.lock().unwrap();
        data.clear();
        data.push_str(rust.to_str().unwrap());
        data.push(char::from(0));
    }
}
#[no_mangle]
pub extern "C" fn javascript_read_stats() -> *const c_char {
    unsafe {
        let xx = DATA_STATS.lock().unwrap();
        let got = CStr::from_bytes_with_nul_unchecked(xx.as_bytes());
        let on_heap = Box::new(got);
        return on_heap.as_ptr();
    }
}
#[cfg(target_os = "emscripten")]
extern "C" {
    pub fn start_javascript_play_sound(sound_id: i32) -> i32;
}

#[cfg(target_os = "emscripten")]
extern "C" {
    pub fn start_game() -> i32;
}

#[cfg(target_os = "emscripten")]
extern "C" {
    pub fn end_game() -> i32;
}

