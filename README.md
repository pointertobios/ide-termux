# IDE termux

为安卓*termux*界面定制的集成开发工具。

## 编译

* 在rust支持本地编译的平台上本地编译

```bash
cargo build --release
```

* 交叉编译到适用于termux的安卓平台

```bash
cargo build --target aarch64-linux-android --release
```
