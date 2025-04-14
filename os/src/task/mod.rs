//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;
use crate::mm::VirtAddr;
use crate::mm::PTEFlags;
use crate::loader::{get_app_data, get_num_app};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    pub inner: UPSafeCell<TaskManagerInner>,
}

/// The task manager inner in 'UPSafeCell'
pub struct TaskManagerInner {
    /// task list
      tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
      current_task: usize,
      /// task time
    pub map: Vec<Vec<(isize,isize)>>
}

lazy_static! {
    /// a `TaskManager` global instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        let mut map=Vec::new();
        for _ in 0..num_app{
            map.push(Vec::new());
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    map,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch4, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let next_task = &mut inner.tasks[0];
        next_task.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut _, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }
    fn push_task_trace(&self,id:isize){
    let mut inner = self.inner.exclusive_access();
    let current = inner.current_task;
    if let Some(v)=inner.map[current].iter_mut().find(|k|k.0==id){
      v.1+=1;
    }else{
        inner.map[current].push((id,1));
    }   
   }
   fn get_task_trace(&self,id:isize)->isize{
    let inner = self.inner.exclusive_access();
    let current = inner.current_task;
    let trace=inner.map[current].clone();
    if let Some(v)=trace.into_iter().find(|k|k.0==id){
        v.1
    }else{
       0
    }
   }
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

/// Change the current 'Running' task's program break
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}
/// 获取id在当前task的计数
pub fn get_task_trace(id:usize)->isize{
    TASK_MANAGER.get_task_trace(id.try_into().unwrap()) 
}
/// 添加id在当前task的计数
pub fn push_task_trace(id:usize){
    TASK_MANAGER.push_task_trace(id.try_into().unwrap())
}
/// 在适当位置添加这个函数 mmap 分配
pub fn task_mmap(start: usize, len: usize, port: usize) -> isize{
    if len == 0 {
        return 0;
    }
    //把start 转成虚拟地址
    let va_start: VirtAddr =VirtAddr::from(start);
    if(!va_start.aligned())||(port & !0x7 != 0) || (port & 0x7 == 0) {
        return -1;
    }
    //结尾的虚拟地址
    let va_end: VirtAddr = VirtAddr::from(start + len);
    let (readable,wraiteable,excuteable)=(port & 0x1,port & 0x2,port & 0x4);
            let mut flags = PTEFlags::V | PTEFlags::U;
            if readable!=0{
                flags |= PTEFlags::R;
            }
          if wraiteable !=0{
                flags |= PTEFlags::W;
            }
          if excuteable!=0{
                flags |= PTEFlags::X;
          }
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
  
    // 获取当前任务的内存集
    let  memory_set =  &mut inner.tasks[current].memory_set;
    
    // 调用内存集的 mmap 方法
    //虚拟地址转页号
    memory_set.mmap(va_start.floor(), va_end.ceil(), flags)
}
///mmap 解散
pub fn task_munmap(start: usize, len: usize) -> isize {
    let va_start: VirtAddr = VirtAddr::from(start);
    if !va_start.aligned(){
        return -1
    }
    // 获取当前任务的内存集
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let  memory_set =  &mut inner.tasks[current].memory_set;
    
    // 调用内存集的 unmmap 方法
    let va_end: VirtAddr = VirtAddr::from((start + len));
    memory_set.unmmap(va_start.floor(),va_end.ceil())
}