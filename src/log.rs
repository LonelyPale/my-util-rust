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

pub fn init_logger(log_mode: LogMode, log_level: tracing::Level) {
    match log_mode {
        LogMode::Original => {
            init_logger_original(log_level);
            return;
        }
        LogMode::Simple => init_logger_simple(log_level),
        LogMode::General => init_logger_general(log_level),
        LogMode::Full => init_logger_full(log_level),
        LogMode::Custom => init_logger_custom(log_level),
    }

    // 设置标准库 `log` 记录器，以便 `tracing` 可以接收 `log` 事件
    // tracing_log::LogTracer::init().expect("Failed to set standard library logger");
    tracing_log::LogTracer::builder()
        .with_max_level(tracing_core::LevelFilter::current().as_log())
        .init().expect("Failed to set standard library logger");
}

fn init_logger_simple(log_level: tracing::Level) {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(log_level)
        .with_writer(std::io::stdout)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default logger");
}

fn init_logger_general(log_level: tracing::Level) {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(log_level)
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_line_number(true)
        .compact()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default logger");
}

fn init_logger_full(log_level: tracing::Level) {
    // let log_level = tracing::Level::TRACE.into();
    // let log_level = tracing::Level::DEBUG.into();
    // let log_level = tracing::Level::INFO.into();

    let filter_layer = tracing_subscriber::EnvFilter::from_default_env().add_directive(log_level.into());
    let tracing_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_thread_ids(true)
        .pretty();

    let collector = tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_layer)
        .with(tracing_error::ErrorLayer::default());

    tracing::subscriber::set_global_default(collector).expect("Could not set global default logger");
}

fn init_logger_custom(log_level: tracing::Level) {
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

/// # runtime error:
/// ```
/// tracing_subscriber::fmt().init();
/// tracing_log::LogTracer::init().expect("TODO: panic message");
/// // tracing_subscriber::fmt().init() 内部已包含 tracing_log::LogTracer::init()，无需再次启动
/// // 二者同时使用有冲突(使用tracing::subscriber::set_global_default()则没有问题)，运行时报错如下：
/// // Message:  Unable to install global subscriber: SetLoggerError(())
/// ```
fn init_logger_original(log_level: tracing::Level) {
    // tracing_subscriber::fmt::init(); //default Level::INFO
    let _logger = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .compact() //紧凑模式
        // .pretty() //美观模式
        .init();
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
