# @wbytts/byi npm 包

这个目录用于把仓库根目录已经构建好的 Rust CLI 产物打包成 npm 包。

## 用法

先在仓库根目录构建 Rust 二进制：

```bash
cargo build --release
```

如果需要一起发布多个平台，也可以预先构建对应 target：

```bash
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-pc-windows-msvc
```

然后进入发布目录执行：

```bash
npm pack
npm publish
```

`prepack` 会自动扫描仓库根目录 `target/` 下已有的 `byi` / `byi.exe`，复制到当前 npm 包的 `dist/` 中。

CI 场景下也支持从指定目录收集各平台二进制：

```bash
BYI_PKG_SOURCE_DIR=/path/to/artifacts npm pack
```

正式发布时，建议先保证：

- Git tag 版本和当前 `package.json` 的 `version` 一致
- GitHub Actions 已收集到各平台 runner 的构建产物
