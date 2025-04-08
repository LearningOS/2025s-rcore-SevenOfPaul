//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// trace syscall
const SYSCALL_TRACE: usize = 410;
extern crate alloc;
use alloc::{collections::BTreeMap, sync::Arc};
use spin::Mutex;
use lazy_static::lazy_static;
mod fs;
mod process;

use fs::*;
use process::*;
lazy_static! {
    static ref SYS_MAP: Arc<Mutex<BTreeMap<usize, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
}
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    let mut map = SYS_MAP.lock();
    if let Some(v) = map.get_mut(&syscall_id) {
        *v += 1;
    } else {
        map.insert(syscall_id, 1);
    }
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => {
            let res=sys_trace(args[0], args[1], args[2],map);
            println!("{:?}结果",res);
            res
        },
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
