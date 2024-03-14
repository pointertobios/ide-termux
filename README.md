# IDE termux

为安卓*termux*界面定制的集成开发工具。

## 编译

* 在windows上编译

```bash
cargo build --target x86_64-pc-windows-msvc
```

* 如果你用的是特别配置的gnu的工具链

```bash
cargo build --target x86_64-pc-windows-gnu
```

* linux

```bash
cargo build --target x86_64-unknown-linux-gnu
```

或

```bash
cargo build --target aarch64-unknown-linux-gnu
```

* 交叉编译到适用于termux的安卓平台

```bash
cargo build --target aarch64-linux-android
```
