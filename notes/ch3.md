
## 任务Task
一个计算阶段的执行过程（也是一段控制流）称为一个 `任务` 。

- 任务运行状态：任务从开始到结束执行过程中所处的不同运行状态：未初始化、准备执行、正在执行、已退出
- 任务控制块：管理程序的执行过程的任务上下文，控制程序的执行与暂停
- 任务相关系统调用：应用程序和操作系统直接的接口，用于程序主动暂停 `sys_yield` 和主动退出 `sys_exit`


## 函数调用规约
RISC-V 架构上的 C 语言调用规范可以在 [这里](https://riscv.org/wp-content/uploads/2015/01/riscv-calling.pdf) 找到。 它对通用寄存器的使用做出了如下约定：

| 寄存器组 | 保存者       | 功能                                                 |
| -------- | ------------ | ---------------------------------------------------- |
| a0~a7    | 调用者保存   | 用来传递输入参数。其中的 a0 和 a1 还用来保存返回值。 |
| t0~t6    | 调用者保存   | 作为临时寄存器使用，在函数中可以随意使用无需保存。   |
| s0~s11   | 被调用者保存 | 作为临时寄存器使用，保存后才能在函数中使用。         |

剩下的 5 个通用寄存器情况如下：

- zero(x0) 之前提到过，它恒为零，函数调用不会对它产生影响；
- ra(x1) 是调用者保存的，不过它并不会在每次调用子函数的时候都保存一次，而是在函数的开头和结尾保存/恢复即可。虽然 `ra` 看上去和其它被调用者保存寄存器保存的位置一样，但是它确实是调用者保存的。
- sp(x2) 是被调用者保存的。这个是之后就会提到的栈指针寄存器。
- gp(x3) 和 tp(x4) 在一个程序运行期间都不会变化，因此不必放在函数调用上下文中。它们的用途在后面的章节会提到。

## 任务上下文
- 任务调度是以函数调用的形式执行的，进入调度函数switch后，ra会保存要回去的指令地址，因此上下文中应保存ra
- 除此之外，因为任务调度是函数调用，因此应该保存callee-saved寄存器。 `s0` - `s11`

```rust
/// TaskContext
/// args:
///     ra  用于保存ret位置
///     s   s0-s11寄存器是callee-saved寄存器，由于switch相当于一个函数调用，因此只需保存callee-saved寄存器
pub struct TaskContext {
    ra: usize,
    s: [usize; 12],
}
```

## 多道程序调度机制
- 协作式调度 (Cooperative Scheduling)
    - 任务主动让出CPU (yield)
- 抢占式调度 (Preemptive Scheduling) 
    - 时间片轮转调度 (Round-Robin Scheduling)

评价指标：
- 性能（Thoughtput & Latency ）
- 公平性 Fairness

## RV64中的中断

参考原文：
[RISC-V 架构中的中断](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/4time-sharing-system.html#risc-v)

中断Interrupt类似Trap，都属于异常ECF。

对于某个处理器核而言， `Trap`与发起`Trap`的指令执行是`同步`的，`Trap`被触发的原因一定能够追溯到某条指令的执行；而`Int`则`异步`(Asynchronous)于当前正在进行的指令，也就是说中断来自于哪个外设以及中断如何触发完全与处理器正在执行的当前指令无关。

也就是说，Trap是同步的，可以从执行指令中推断何时执行；Int是异步的，无法确定执行的时间，也就是说，中断的随机性。

> 硬件角度理解同步与异步
> - 处理器内部有各种function units
> - Trap因为指令的执行而引起，因此在这些units内部就可以发现Trap, 发起Trap的电路也在这内部
> - Int是一套与处理器执行指令无关的一套电路发起的，发起电路和这些units说并行的，只有一根导线连接

### RV64中断表：

- **软件中断** (Software Interrupt)：由软件控制发出的中断
- **时钟中断** (Timer Interrupt)：由时钟电路发出的中断
- **外部中断** (External Interrupt)：由外设发出的中断

| Interrupt | Exception Code | Description                   |
| --------- | -------------- | ----------------------------- |
| 1         | 1              | Supervisor software interrupt |
| 1         | 3              | Machine software interrupt    |
| 1         | 5              | Supervisor timer interrupt    |
| 1         | 7              | Machine timer interrupt       |
| 1         | 9              | Supervisor external interrupt |
| 1         | 11             | Machine external interrupt    |

每个中断都有S和M模式的的两个版本。中断的特权级可以决定该中断是否会被屏蔽，以及需要到 CPU 的哪个特权级进行处理。

## 中断的屏蔽
- 高特权级下收到低特权级的中断，会被屏蔽
- 收到当前特权级的中断，则需要通过相应的 CSR 判断该中断是否会被屏蔽。

以内核所在的 S 特权级的中断屏蔽相关CSR：
- sstatus: sstatus 的 sie 为 S 特权级的中断使能，能够同时控制三种中断，如果将其清零则会将它们全部屏蔽。
- sie: sstatus.sie 置 1 时，需要根据这个寄存器判断
    - ssie: 控制S特权级的软件中断
    - stie: 控制S特权级的时钟中断
    - seie: 控制S特权级的外部中断

## 中断的处理
如果中断没有被屏蔽，那么接下来就需要软件进行处理，和一些CSR有关。
- 默认所有的中断都需要到 M 特权级处理。
- 修改一些CSR后，就可以让中断打到低特权级处理。但是打到的特权级不能低于中断的特权级。（比如不能把M的中断打到S处理）

默认情况下，当中断产生并进入某个特权级之后，在中断处理的过程中同特权级的中断都会被屏蔽：
- 中断发生时，sstatus的sie会保存到spie，sie置零，屏蔽同级中断
- sret后会会恢复之前保存的sie

默认情况下不会嵌套

## 时钟中断

- mtime:  64 位的 CSR, 用于记录自系统启动以来经过的时钟周期数
- mtimecmp: 一旦计数器 mtime 的值超过了 mtimecmp，就会触发一次时钟中断。

这两个CSR都是M模式的CSR， SEE（RustSBI）有相关接口。