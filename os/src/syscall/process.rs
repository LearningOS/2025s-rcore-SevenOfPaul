//! Process management syscalls
use crate::{mm::{PageTable, VirtAddr}, task::{change_program_brk, current_user_token, exit_current_and_run_next, get_task_trace, suspend_current_and_run_next}, timer::get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    //这里也一样
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    unsafe {
        let cur_token=current_user_token();
        let page_table=PageTable::from_token(cur_token);
        //获取实际地址
        let va=VirtAddr::from(_id);
        let page_num=va.floor();
        let offset=va.page_offset();
         let pte=page_table.translate(page_num).unwrap();
         let ppn=pte.ppn();
         //ppa是实际地址
         let ppa=ppn.0+offset;
            match _trace_request {
            //这里需要把用户的虚拟地址改为物理地址
            0 =>{
                if pte.is_valid()&&pte.readable(){
                    *(ppa as *const u8) as isize
                }else{
                    -1
                }
              
            },
            1 => {
                if pte.is_valid()&&pte.writable(){
                    {*(ppa as *mut u8) = _data as u8};
                    0
                }else{
                    -1
                }   
            }
            2 => get_task_trace(ppa),
            _ => -1,
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
