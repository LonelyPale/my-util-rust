/// 打印 eyre error 和 panic 时，美化输出
/// 
/// 打印调用栈时，只打印以`package_name`开头的记录，如果`package_name=""`则打印全部
/// 
/// # Example
/// ```
/// let err = eyre::eyre!("error: test");
/// panic!("1 {err}");
/// panic!("2 {err:?}");
/// panic!("3 {err:#}");
/// panic!("4 {err:#?}");
/// ```
pub fn init_error_hook(package_name: &'static str) {
    // color_eyre::install().unwrap();
    color_eyre::config::HookBuilder::default()
        .add_frame_filter(Box::new(move |frames| {
            let filters = &[package_name];

            //过滤调用栈
            frames.retain(|frame| {
                // tracing::debug!("{}", frame.name.as_ref().unwrap());

                filters.iter().any(|filter| {
                    if let Some(name) = frame.name.as_ref() {
                        let name = name.as_str();
                        name.starts_with(filter)
                    } else {
                        true
                    }
                })
            });
        }))
        .display_location_section(false) //表示在错误报告中是否显示错误发生的具体代码位置信息，这不会禁用紧急消息中的位置部分。
        .display_env_section(false) //表示在错误报告中是否显示环境信息部分。
        .install()
        .expect("Failed to initialize color_eyre");
}

#[cfg(test)]
mod tests {
    use eyre::{Report, Result, WrapErr};
    use crate::error::*;

    /// 在Rust中，如果你想要在`println!`宏中输出花括号字符"{}"，你可以使用双花括号"{{"和"}}"来转义它们。这是因为在`println!`宏中，花括号"{}"用于格式化输出，而"{"和"}"被认为是特殊字符。因此，如果你想要输出花括号字符本身，你需要将它们用双花括号包裹起来，如下所示：
    ///
    /// ```rust
    /// fn main() {
    ///     println!("Hello, {{}}"); // 输出: Hello, {}
    /// }
    /// ```
    ///
    /// 这样做会使得`println!`宏输出的文本中包含实际的花括号字符"{}"，而不会被解释为格式化输出的一部分。
    fn print_error(err: &Report) {
        println!("{{err}} >> {err}");
        println!("{{err:?}} >> {err:?}");
        println!("{{err:#}} >> {err:#}");
        println!("{{err:#?}} >> {err:#?}");
    }

    fn my_err() -> Report {
        let err = || -> Result<()> {
            Err(eyre::eyre!("error: my error 1"))
        }().context("my error 2").context("my error 3").unwrap_err();

        err
    }

    #[test]
    fn error_no_hook_test() {
        let err = my_err();
        print_error(&err);
        panic!("panic: {err:?}");
    }

    #[test]
    fn error_hook_test() {
        let package_name = "myutil";
        init_error_hook(package_name);

        let err = my_err();
        print_error(&err);
        panic!("panic: {err:?}");
    }
}
