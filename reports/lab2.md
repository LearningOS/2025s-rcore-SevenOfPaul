1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

 群里的各位大佬

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容： 
+ mmap 可以不是连续的空间 所以可以使用btreemap 来存储
+ 同时 每个应用都有自己独立的memorySet 所以在memorySet中保存一份 key为虚拟页号 val为
物理页帧的数据结构
+ 所以需要只要维护一份start-end的虚拟页号以及对应的页帧即可
```
let va_start: VirtAddr =VirtAddr::from(start);//虚拟地址
va_start.floor();//页号保存在虚拟地址的最后一位
let va_end: VirtAddr = VirtAddr::from(start+len);//虚拟地址
va_end.ceil();//页号保存在虚拟地址的第一位
``` 
+ 现在已经获取到了start和end的页号 下面可以使用btreemap来存储
```
while va_start != va_end {
    if let Some(frame_tracker)=frame_alloc{
        //还需要在页表中进行映射虚拟地址和物理页号
        self.page_table.map(va_start, frame_tracker.ppn);
      //保存虚拟页号和对应的物理页帧
    btreemap.insert(va_start,frame_tracker);
    }
    //通过以提供的迭代器+1
    va_start.step();
}
```
+ 解除分配正相反
```
while va_start != va_end {
    if let Some(frame_tracker)=frame_alloc{
        //删除映射关系
        self.page_table.unmap(va_start);
      //删除btreemap中的虚拟页号
    btreemap.remove(va_start);
    }
    //通过以提供的迭代器+1
    va_start.step();
}
```
我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。