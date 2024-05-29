# rust project
当`cargo init --lib .`未设置`name`时，使用当前文件夹名称作为`package.name`。

当`cargo init --lib --name myutil .`设置`name`时，则使用`name`作为`package.name`。

```shell
cargo init --lib .
cargo init --lib --name myutil .
```

```toml
[package]
name = "my-util-rust"
version = "0.1.0"
edition = "2021"
```

当`lib.name`未设置时，使用`package.name`作为依赖项用来引用它的包名称，任何`-`破折号都将替换为`_`下划线。 也就是如果`package.name`中包含`-`破折号，则需要在coding中导入包`mod`或`use`时，把`-`破折号替换为`_`下划线。

当`lib.name`设置有效值时，忽略`package.name`，使用`lib.name`作为依赖项用来引用它的包名称。

```toml
[lib]
name = "myutil"
```

## reference 参考
[Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)

[cargo-init(1)](https://doc.rust-lang.org/cargo/commands/cargo-init.html)

[The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)

[Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html)


# dependencies 依赖
```shell
# 错误处理
cargo add --optional eyre
cargo add --optional color-eyre

#日志处理: tracing_log用于兼容标准库的log
cargo add --optional tracing
cargo add --optional -F env-filter,chrono tracing-subscriber
cargo add --optional --features env-filter,chrono tracing-subscriber
cargo add --optional --features "env-filter chrono" tracing-subscriber
cargo add --optional tracing-error
cargo add --optional tracing-core
cargo add --optional tracing-log

#日期时间
cargo add --optional chrono
cargo remove chrono

```

## reference 参考
[Command-line feature options](https://doc.rust-lang.org/cargo/reference/features.html#command-line-feature-options)
[Optional dependencies](https://doc.rust-lang.org/cargo/reference/features.html#optional-dependencies)
