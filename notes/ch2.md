## rv64优先级
| 级别 | 编码 | 名称                                |
| ---- | ---- | ----------------------------------- |
| 0    | 00   | 用户/应用模式 (U, User/Application) |
| 1    | 01   | 监督模式 (S, Supervisor)            |
| 2    | 10   | 虚拟监督模式 (H, Hypervisor)        |
| 3    | 11   | 机器模式 (M, Machine)               |

## Exception Control Flow
在 RISC-V 架构中，这种与常规控制流 （顺序、循环、分支、函数调用）不同的 异常控制流 (ECF, Exception Control Flow) 被称为 异常（Exception）。

用户态应用直接触发从用户态到内核态的 **异常控制流** 的原因总体上可以分为两种：执行 `Trap类异常` 指令和执行了会产生 `Fault类异常` 的指令 。`Trap类异常` 指令 就是指用户态软件为获得内核态操作系统的服务功能而发出的特殊指令。 `Fault类` 的指令是指用户态软件执行了在内核态操作系统看来是非法操作的指令。下表中我们给出了 RISC-V 特权级定义的会导致从低特权级到高特权级的各种 **异常**：

- 执行Trap类异常指令
- 执行会产生Fault类异常的指令

RISC-V 异常一览表
| Interrupt | Exception Code | Description                    |
| --------- | -------------- | ------------------------------ |
| 0         | 0              | Instruction address misaligned |
| 0         | 1              | Instruction access fault       |
| 0         | 2              | Illegal instruction            |
| 0         | 3              | Breakpoint                     |
| 0         | 4              | Load address misaligned        |
| 0         | 5              | Load access fault              |
| 0         | 6              | Store/AMO address misaligned   |
| 0         | 7              | Store/AMO access fault         |
| 0         | 8              | Environment call from U-mode   |
| 0         | 9              | Environment call from S-mode   |
| 0         | 11             | Environment call from M-mode   |
| 0         | 12             | Instruction page fault         |
| 0         | 13             | Load page fault                |
| 0         | 15             | Store/AMO page fault           |

其中Trap有两种，都是主动触发的
- Breakpoint
    - 由ebreak触发
- Environment call
    - 由ecall触发
    - 不同的特权级执行时会触发不同的trap
    - SEE和OS之间的接口称为SBI
    - 用户程序和OS之间的接口称为ABI，更通常称为syscall


其他异常都是在访问指令的时候，发生了某种错误，都是`Fault`。

## rv中的特权指令
特权级无关的一般的指令 和 通用寄存器 x0~x31 在任何特权级都能访问。

每个特权级都对应一些特殊指令和 控制状态寄存器 (CSR, Control and Status Register) ，来控制该特权级的某些行为并描述其状态。当然特权指令不仅具有读写 CSR 的指令，还有其他功能的特权指令。

越权访问会出现Illegal instruction。

S模式的特权指令
- 本身具有特权的指令，比如sret
- 指令访问了 S模式特权级下才能访问的寄存器 或内存，如表示S模式系统状态的 控制状态寄存器 sstatus 等。

## 特权级切换
---
S 特权级 Trap 的相关 CSR
| CSR 名  | 该 CSR 与 Trap 相关的功能                                    |
| ------- | ------------------------------------------------------------ |
| sstatus | `SPP` 等字段给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息 |
| sepc    | 当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址 |
| scause  | 描述 Trap 的原因                                             |
| stval   | 给出 Trap 附加信息                                           |
| stvec   | 控制 Trap 处理代码的入口地址                                 |

### Trap处理函数
在U mode下通过ecall可以触发S mode的Trap。

stvec中的地址说明了中断处理函数所在的地址。

> **stvec 相关细节**
> 在 RV64 中， `stvec` 是一个 64 位的 CSR，在中断使能的情况下，保存了中断处理的入口地址。它有两个字段：
> - MODE 位于 [1:0]，长度为 2 bits；
> - BASE 位于 [63:2]，长度为 62 bits。
> 
>  当 MODE 字段为 0 的时候， `stvec` 被设置为 Direct 模式，此时进入 S 模式的 Trap 无论原因如何，处理 Trap 的入口地址都是 `BASE<<2` ， CPU 会跳转到这个地方进行异常处理。

本项目主要是用Direct模式，因此直接将中断处理函数的地址放入svec中，即可。（要求中断处理函数4字节对齐，最后两位就是0）

### 特权级降级（回到U mode）
而当 CPU 完成 Trap 处理准备返回的时候，需要通过一条 S 特权级的特权指令 `sret` 来完成，这一条指令具体完成以下功能：

- CPU 会将当前的特权级按照 `sstatus` 的 `SPP` 字段设置为 U 或者 S 
- CPU 会跳转到 `sepc` 寄存器指向的那条指令，然后继续执行。


### trap上下文
- ecall时，自动保存sepc为ecall处的地址。
- trap处理函数中可能会修改通用寄存器x0-x31
- trap处理函数中可能修改sstatus，用于指定sret回去的特权级

因此trap上下文要保存这些东西。
