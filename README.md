# rnix-rv64 
## About
基于 [rcore-os](https://github.com/rcore-os/rCore-Tutorial-v3) 的类unix操作系统内核。算是 [rnix-x86](https://github.com/RockRockWhite/rnix-x86) 的延续。

运行ISA：RISC-V 64

## 进度安排
- 前期参考RCore先书写基本内容
- 后期会自己在这个基础上写一些有意思的东西

## 进度流水账
---
- [x] 进入内核
- [x] 批处理操作系统
- [x] 多道程序
- [x] 任务协作调度yield
- [x] 任务抢占调度
---
### 批处理系统 batch system
![image-20230630135101165](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/image-20230630135101165.png)

### 多道程序协作式调度
![image-20230708181206947](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/image-20230708181206947.png)

![image-20230710151018052](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/image-20230710151018052.png)

### 多道程序抢占式调度 Round Robin
![image-20230710215211482](https://typora-1303830133.cos.ap-shanghai.myqcloud.com/typora/img/image-20230710215211482.png)