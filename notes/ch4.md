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

## 页表