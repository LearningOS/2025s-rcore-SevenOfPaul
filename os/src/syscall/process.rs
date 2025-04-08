//! Process management syscalls
use alloc::collections::btree_map::BTreeMap;
use spin::MutexGuard;

use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

#[deny(warnings)]
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize,map:MutexGuard<'_, BTreeMap<usize, usize>>) -> isize {
   match _trace_request {
        0 => {
         unsafe { *(_id as *const u8) as isize }
        },
        1 => {
            unsafe {
                *(_id as *mut u8) = _data as u8;
            }
            0
        },
        2 => {
            *map.get(&_id).unwrap() as isize
        },
        _ => -1
    }
}
