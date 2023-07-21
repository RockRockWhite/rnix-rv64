## 地址空间相关映射
- 内存管理单元 (MMU, Memory Management Unit) 
    - 负责执行指令的时候自动将虚拟地址转换为物理地址
    - MMU 可能会将来自不同两个应用地址空间的相同虚拟地址转换成不同的物理地址，需要硬件来提供寄存器管理映射关系

![../_images/page-table.png](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/page-table.png)

## 地址映射相关CSR
默认情况下 MMU 未被使能，此时无论 CPU 位于哪个特权级，访存的地址都会作为一个物理地址交给对应的内存控制单元来直接 访问物理内存。

修改S特权级的`satp` CSR, 可以让S和U特权级下访存启用MMU。

M特权级下的访存可以设置成直接是物理地址。

![../_images/satp.png](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/satp.png)
- `MODE` 控制 CPU 使用哪种页表实现。
    - 为0时，所有访存都是物理地址
    - 为8时，启用了SV39分页机制，对S/U生效
- `ASID` 表示地址空间标识符，这里还没有涉及到进程的概念，我们不需要管这个地方。
- `PPN` 存的是根页表所在的物理页号。这样，给定一个虚拟页号，CPU 就可以从三级页表的根页表开始一步步的将其映射到一个物理页号。


## 地址格式
![../_images/sv39-va-pa.png](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/sv39-va-pa.png)

SV39中会把39位VA转化成56位PA，SV39 分页模式规定64位虚拟地址的高25位必须和第38位相同，否则不是合法的虚拟地址。也就是，在64位的地址空间中，只有最高的和最低的256GB是合法的VA，其他的地址都是不合法的。

## 页表项 PTE Page Table Entry
![../_images/sv39-pte.png](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/sv39-pte.png)

上图为 SV39 分页模式下的页表项，其中 [53:10] 这 44 位是物理页号，最低的 8 位 [7:0] 则是标志位，它们的含义如下（请注意，为方便说明，下文我们用 *页表项的对应虚拟页面* 来表示索引到 一个页表项的虚拟页号对应的虚拟页面）：

- V(Valid)：仅当位V为 1 时，页表项才是合法的；
- R(Read)/W(Write)/X(eXecute)：分别控制索引到这个页表项的对应虚拟页面是否允许读/写/执行；
- U(User)：控制索引到这个页表项的对应虚拟页面是否在 CPU 处于 U 特权级的情况下是否被允许访问；
- G：暂且不理会；
- A(Accessed)：处理器记录自从页表项上的这一位被清零之后，页表项的对应虚拟页面是否被访问过；
- D(Dirty)：处理器记录自从页表项上的这一位被清零之后，页表项的对应虚拟页表是否被修改过。

除了 `G` 外的上述位可以被操作系统设置，只有 `A` 位和 `D` 位会被处理器动态地直接设置为 `1` ，表示对应的页被访问过或修改过。

