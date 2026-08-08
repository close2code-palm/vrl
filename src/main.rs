use clap::Parser;
use vrl::cli::{
    Opts,
    cmd::{Command, cmd, fmt_cmd},
};

fn main() {
    let opts = Opts::parse();
    let status = match &opts.command {
        Some(Command::Fmt(fmt_opts)) => fmt_cmd(fmt_opts),
        None => cmd(&opts, vrl::stdlib::all()),
    };
    std::process::exit(status);
}
