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

## 多级页表
SV39分页机制相当于一个字典树，9 * 3 = 27， 看成是3位字符，每一位9个bit。

最后一级页表保存 $2^9 = 512$个PTE，每个PTE有64bit，8个Byte， 512 * 8 = 4KB，正正好可以放到一个物理页内！

非叶子结点的PET定义与叶子结点不同:
- 当 `V` 为 0 的时候，代表当前指针是一个空指针，无法走向下一级节点，即该页表项对应的虚拟地址范围是无效的；
- 只有当 `V` 为1 且 `R/W/X` 均为 0 时，表示是一个合法的页目录表项，其包含的指针会指向下一级的页表；
- 注意: 当`V` 为1 且 `R/W/X` 不全为 0 时，表示是一个合法的页表项，其包含了虚地址对应的物理页号。即`大页`。

![img](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/pte-rwx.png)

## SV39 Address Translation
![img](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/sv39-full.png)

假设我们有虚拟地址 (VPN2,VPN1,VPN0,offset) ：

- 我们首先会记录装载「当前所用的三级页表的物理页」的页号到 satp 寄存器中；
- 把 VPN2 作为偏移在第三级页表的物理页中找到第二级页表的物理页号；
- 把 VPN1 作为偏移在第二级页表的物理页中找到第一级页表的物理页号；
- 把 VPN0 作为偏移在第一级页表的物理页中找到要访问位置的物理页号；
- 物理页号对应的物理页基址（即物理页号左移12位）加上 offset 就是虚拟地址对应的物理地址。

## 快表TLB Translation Lookaside Buffer

`satp`寄存器中PPN指向多级页表根节点所在的物理页号。TLB是多几页表的Cache，用来加速地址转换。

我们手动修改一个页表项之后，也修改了映射，但 TLB 并不会自动刷新清空，我们也需要使用 sfence.vma 指令刷新整个 TLB。注：可以在 sfence.vma 指令后面加上一个虚拟地址，这样 sfence.vma 只会刷新TLB中关于这个虚拟地址的单个映射项。

