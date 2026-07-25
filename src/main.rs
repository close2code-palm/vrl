use clap::Parser;
use vrl::cli::{
    Opts,
    cmd::{FmtOpts, cmd, fmt},
};

fn main() {
    let args = std::env::args_os().collect::<Vec<_>>();
    if args.get(1).and_then(|arg| arg.to_str()) == Some("fmt") {
        let fmt_args = std::iter::once(args[0].clone())
            .chain(args.into_iter().skip(2))
            .collect::<Vec<_>>();
        let opts = FmtOpts::parse_from(fmt_args);
        let status = match fmt(&opts) {
            Ok(true) if opts.check => {
                #[allow(clippy::print_stderr)]
                eprintln!("{} needs formatting", opts.file.display());
                exitcode::DATAERR
            }
            Ok(_) => exitcode::OK,
            Err(error) => {
                #[allow(clippy::print_stderr)]
                eprintln!("{error}");
                exitcode::SOFTWARE
            }
        };
        std::process::exit(status);
    }

    std::process::exit(cmd(&Opts::parse(), vrl::stdlib::all()));
}
