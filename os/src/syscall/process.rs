//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{PageTable, VirtAddr, VirtPageNum},
    task::{TASK_MANAGER,
        change_program_brk, current_user_token, exit_current_and_run_next, get_task_trace,
        suspend_current_and_run_next,
    },
    timer::get_time_us,
};

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
    let time = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    //这里也一样
    let page_table = PageTable::from_token(current_user_token());

    // 获取虚拟地址
    let va = VirtAddr::from(ts as usize);
    let vpn = va.floor();
    let offset = va.page_offset();
    let pte = page_table.translate(vpn).unwrap();
    let size_of_timeval = core::mem::size_of::<TimeVal>();
    let ppn = pte.ppn();
    let ppa = (ppn.0 << 12) + offset;
    //如果size_of_timeval大于4096，那么就需要两个页来存储
    if offset + size_of_timeval <= PAGE_SIZE {
        unsafe { *(ppa as *mut TimeVal) = time }
    } else {
        let bytes = unsafe {
            core::slice::from_raw_parts(&time as *const TimeVal as *const u8, size_of_timeval)
        };
        let f_page = PAGE_SIZE - offset;
        for i in 0..f_page {
            unsafe { *((ppa + i) as *mut u8) = bytes[i] }
        }
        let vpn2 = VirtPageNum(vpn.0 + 1);
        let pte2 = page_table.translate(vpn2).unwrap();
        let ppn2 = pte2.ppn();
        let pa2 = ppn2.0 << 12;
        for i in 0..(bytes.len() - f_page) {
            unsafe { *((pa2 + i) as *mut u8) = bytes[f_page + i] }
        }
    }

    0
}
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if len == 0 {
        return 0;
    }
    if (start % PAGE_SIZE != 0) || (port & !0x7 != 0) || (port & 0x7 == 0) {
        return -1;
    }
    //这种实现过于松散 下午整合一下
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
  
    // 获取当前任务的内存集
    let  memory_set =  &mut inner.tasks[current].memory_set;
    
    // 调用内存集的 mmap 方法
    memory_set.mmap(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if len == 0 {
        return 0;
    }
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    
    // 获取当前任务的内存集
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let  memory_set =  &mut inner.tasks[current].memory_set;
    
    // 调用内存集的 unmmap 方法
    memory_set.unmmap(start, len)
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
         if let Some(pte)=page_table.translate(page_num){
         let ppn=pte.ppn();
         //ppa是实际地址
         let ppa: usize=(ppn.0<<12)+offset;
            match _trace_request {
            //这里需要把用户的虚拟地址改为物理地址
            0 =>{
                if pte.is_valid()&&pte.readable()&&pte.is_user(){
                    *(ppa as *const u8) as isize
                }else{
                    -1
                }
              
            },
            1 => {
                if pte.is_valid()&&pte.writable()&&pte.is_user(){
                    {*(ppa as *mut u8) = _data as u8};
                    0
                }else{
                    -1
                }   
            }
            2 => {
                get_task_trace(_id)
            },
            _ => -1,
        }
    }else{
        -1
    }
}
}
// // pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
// //    let mut kr= KERNEL_SPACE.exclusive_access();
// //    kr.mmap(start, len, port)
// // }
// // pub fn sys_munmap(start: usize, len: usize) -> isize {
// //     let mut kr= KERNEL_SPACE.exclusive_access();
// //     kr.unmmap(start, len)
// // }
// // YOUR JOB: Implement mmap.
// pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
//     // if len==0{
//     //     return 0;
//     // }
//     if (start % PAGE_SIZE != 0) || (port & !0x7 != 0) || (port & 0x7 == 0) {
//       -1
//     } else {
//         let pages_len = (len + PAGE_SIZE - 1) / PAGE_SIZE;
//         let mut page_table = PageTable::from_token(current_user_token());
//         for i in 0..pages_len{
//             //start是物理地址 除以size就是页号 检测是否有以及被映射的页号
//             let vpn = VirtPageNum((start/PAGE_SIZE) + i);
//             if let Some(pte)=page_table.translate(vpn){
//                 if pte.is_valid(){
//                     return -1;
//                 }
//             }
//         }
//         // let ppn=alloc_pages(pages);
//         let (readable,wraiteable,excuteable)=(port & 0x1,port & 0x2,port & 0x4);
//         let mut flags = PTEFlags::V | PTEFlags::U;
//         if readable!=0{
//             flags |= PTEFlags::R;
//         }
//       if wraiteable !=0{
//             flags |= PTEFlags::W;
//         }
//       if excuteable!=0{
//             flags |= PTEFlags::X;
//       }
//       //准备计算ppn
//          for i in 0..pages_len{
//           if let Some(frame)=frame_alloc(){ 
//            let vpn=VirtPageNum((start/PAGE_SIZE) + i);
//             let ppn=frame.ppn;
//             page_table.map(vpn, ppn, flags);
//             println!("{:?}=={:?}",page_table.translate(vpn).unwrap().is_valid(),vpn);
//           }
//           else{
//             //清理之前
//             // for j in 0..i{
//             //     let vpn=VirtPageNum((start/PAGE_SIZE) + j);
//             //     page_table.unmap(vpn);
//             // }
//             // return -1
//           }
//          }
//         0
//     }
// }

// // YOUR JOB: Implement munmap.
// pub fn sys_munmap(start: usize, len: usize) -> isize {
//     if len == 0 {
//         return 0;
//     }
//     let mut page_table = PageTable::from_token(current_user_token());
//     let pages_len = (len + PAGE_SIZE - 1) / PAGE_SIZE;
//     for i in 0..pages_len {
//         let vpn = VirtPageNum((start/PAGE_SIZE) + i);
//         // 检查页面是否已映射且有效
//         if let Some(pte) = page_table.translate(vpn) {
//             if !pte.is_valid()||!pte.is_user() {
//                 debug!("unmap fail don't aligned");
//                 println!("{:?}=vpn={:?}==page{:?}",!pte.is_valid(),vpn,i);
//                 return -1;
//             }
//             page_table.unmap(vpn);
//         } else {
//             return -1;
//         }
//     }
//     return 0;  
// }
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
