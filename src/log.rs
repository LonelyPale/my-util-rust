use tracing_core::{Event, Subscriber};
use tracing_log::AsLog;
use tracing_subscriber::fmt::{FmtContext, format, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;

pub enum LogMode {
    Original,
    Simple,
    General,
    Full,
    Custom,
}

pub fn init_log(log_mode: LogMode, log_level: tracing::Level) {
    match log_mode {
        LogMode::Original => {
            init_log_original(log_level);
            return;
        }
        LogMode::Simple => init_log_simple(log_level),
        LogMode::General => init_log_general(log_level),
        LogMode::Full => init_log_full(log_level),
        LogMode::Custom => init_log_custom(log_level),
    }

    // 设置标准库 `log` 记录器，以便 `tracing` 可以接收 `log` 事件
    // tracing_log::LogTracer::init().expect("Failed to set standard library logger");
    tracing_log::LogTracer::builder()
        .with_max_level(tracing_core::LevelFilter::current().as_log())
        .init().expect("Failed to set standard library logger");
}

/// # runtime error:
/// ```
/// tracing_subscriber::fmt().init();
/// tracing_log::LogTracer::init().expect("panic message");
/// // tracing_subscriber::fmt().init() 内部已包含 tracing_log::LogTracer::init()，无需再次启动
/// // 二者同时使用有冲突(使用tracing::subscriber::set_global_default()则没有问题)，运行时报错如下：
/// // Message:  Unable to install global subscriber: SetLoggerError(())
/// ```
fn init_log_original(log_level: tracing::Level) {
    // tracing_subscriber::fmt::init(); //default Level::INFO
    let _logger = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .compact() //紧凑模式
        // .pretty() //美观模式
        .init();
}

fn init_log_simple(log_level: tracing::Level) {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(log_level)
        .with_writer(std::io::stdout)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default logger");
}

/// # tracing: local time print `<unknown time>`
///
/// tracing_subscriber 版本 0.3.* 中使用`time`输出自定义时间时错误打印`<unknown time>`，使用`chrono`则无此问题。
///
/// [subscriber: don't bail when timestamp formatting fails #1689](https://github.com/tokio-rs/tracing/pull/1689)
///
/// [tracing_subscriber : The log CAN NOT display the time correctly in the LINUX with tracing_subscriber::fmt().with_timer(LocalTime::rfc_3339()) #2715](https://github.com/tokio-rs/tracing/issues/2715)
///
/// [tracing_subscriber::fmt::time::LocalTime not working when multiple threads #2004](https://github.com/tokio-rs/tracing/issues/2004)
///
/// [unable to get LocalTime on OpenBSD #2764](https://github.com/tokio-rs/tracing/issues/2764)
fn init_log_general(log_level: tracing::Level) {
    // let timer = tracing_subscriber::fmt::time::ChronoLocal::default();
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(log_level)
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_line_number(true)
        .with_timer(timer)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default logger");
}

fn init_log_full(log_level: tracing::Level) {
    // 创建一个Tracing的事件过滤器
    let filter_layer = tracing_subscriber::EnvFilter::from_default_env().add_directive(log_level.into());

    // 创建一个自定义的时间戳格式器
    // let timer = tracing_subscriber::fmt::time::ChronoLocal::default();
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());

    // 创建一个Tracing的格式化器，并设置时间戳格式器
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_timer(timer)
        // .without_time() //不显示时间
        .pretty();

    // 创建一个Tracing订阅器，并将格式化器和事件过滤器添加到其中
    let collector = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(tracing_error::ErrorLayer::default());

    // 使用Tracing订阅器
    tracing::subscriber::set_global_default(collector).expect("Could not set global default logger");
}

fn init_log_custom(log_level: tracing::Level) {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(log_level)
        .with_writer(std::io::stdout)
        // .with_target(true)
        // .with_file(true)
        // .with_line_number(true)
        // .with_thread_names(true)
        // .with_thread_ids(true)
        // .compact()
        // .pretty()
        .event_format(CustomFormatter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default logger");
}

struct CustomFormatter;

/// 自定义 tracing 日志输出格式：
/// https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/trait.FormatEvent.html
impl<S, N> FormatEvent<S, N> for CustomFormatter
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
        N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // Format values from the event's's metadata:
        let metadata = event.metadata();
        write!(&mut writer, "{} {}: ", metadata.level(), metadata.target())?;

        let line = metadata.line().unwrap_or(0);
        let full_path = metadata.file().unwrap_or("unknown");
        let filename = full_path.split('/').last().unwrap_or(full_path);
        let filename_display = if filename.len() > 20 {
            &filename[0..20]
        } else {
            filename
        };
        write!(writer, "filename={filename_display}:{line} -> ")?;

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

#[cfg(test)]
mod tests {
    use eyre::{Context, Report};

    use crate::log::{init_log, LogMode};

    fn my_err() -> Report {
        let err = || -> eyre::Result<()> {
            Err(eyre::eyre!("error: my error 1"))
        }().context("my error 2").context("my error 3").unwrap_err();

        err
    }

    fn display() {
        tracing::trace!("[trace]-1 log info");
        tracing::debug!("[debug]-2 log info");
        tracing::info!("[info]-3 log info");
        tracing::warn!("[warn]-4 log info");
        tracing::error!("[error]-5 log info");
        tracing::error!("[error]-5.1 {}", my_err());
        tracing::error!("[error]-5.2 {:?}", my_err());
        tracing::error!("[error]-5.3 {:#}", my_err());
        tracing::error!("[error]-5.4 {:#?}", my_err());
    }

    #[test]
    fn display_original() {
        init_log(LogMode::Original, tracing::Level::TRACE);
        display();
    }

    #[test]
    fn display_simple() {
        init_log(LogMode::Simple, tracing::Level::TRACE);
        display();
    }

    #[test]
    fn display_general() {
        init_log(LogMode::General, tracing::Level::TRACE);
        display();
    }

    #[test]
    fn display_full() {
        init_log(LogMode::Full, tracing::Level::TRACE);
        display();
    }

    #[test]
    fn display_custom() {
        init_log(LogMode::Custom, tracing::Level::TRACE);
        display();
    }
}
