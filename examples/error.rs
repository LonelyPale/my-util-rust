use eyre::{Context, Report};
use myutil::error::init_error_hook;

fn main() {
    let package_name = "error";
    init_error_hook(package_name);

    let err = my_err();
    print_error(&err);
    panic!("panic: {err:?}");
}

fn print_error(err: &Report) {
    println!("1 {{err}} >> {err}");
    println!("2 {{err:?}} >> {err:?}");
    println!("3 {{err:#}} >> {err:#}");
    println!("4 {{err:#?}} >> {err:#?}");
}

fn my_err() -> Report {
    let err = || -> eyre::Result<()> {
        Err(eyre::eyre!("error: my error 1"))
    }().context("my error 2").context("my error 3").unwrap_err();

    err
}
