//! Process management syscalls
use crate::{config::PAGE_SIZE, mm::{PageTable, VirtAddr, VirtPageNum}, task::{change_program_brk, current_user_token, exit_current_and_run_next, get_task_trace, suspend_current_and_run_next}, timer::get_time_us};

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
    let us = get_time_us();
    let time= TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    //这里也一样
    let page_table = PageTable::from_token(current_user_token());
    
    // 获取虚拟地址
    let va = VirtAddr::from(ts as usize);
    let vpn = va.floor();
    let offset = va.page_offset();
    let pte=page_table.translate(vpn).unwrap();
    let size_of_timeval = core::mem::size_of::<TimeVal>();
    let ppn=pte.ppn();
    let  ppa=(ppn.0<<12)+offset;
    //如果size_of_timeval大于4096，那么就需要两个页来存储
    if offset+size_of_timeval<=PAGE_SIZE{
        unsafe {
            *(ppa as *mut TimeVal) =time
        }
    }else{
       let bytes=unsafe {
           core::slice::from_raw_parts(&time as *const TimeVal  as *const u8, size_of_timeval)
       };
       let f_page=PAGE_SIZE-offset;
       for i in 0..f_page{
           unsafe {
               *((ppa+i) as *mut u8) = bytes[i]
           }
       }
       let vpn2=VirtPageNum(vpn.0+1);
       let pte2=page_table.translate(vpn2).unwrap();
       let ppn2 = pte2.ppn();
       let pa2 = ppn2.0 << 12; 
       for i in 0..(bytes.len()-f_page){
        unsafe {
            *((pa2+i) as *mut u8) = bytes[f_page+i]
        }
    }
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
         let ppa: usize=(ppn.0<<12)+offset;
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
            2 => {
                // 对于系统调用跟踪，应该直接传递系统调用ID而不是物理地址
                get_task_trace(_id)
            },
            _ => -1,
        }
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if start%PAGE_SIZE!=0||port&!0x7 != 0||prot & 0x7 = 0{
 -1
    }else{
           let pages = (len + PAGE_SIZE-1)/PAGE_SIZE;
        let page_table=PageTable::from(current_user_token());
        let ppn=
        page_table.map(VirtPageNum(start), ppn, flags);
        0
    }
   
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
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
