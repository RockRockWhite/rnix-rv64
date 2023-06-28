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