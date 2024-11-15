use std::env;
use std::net::SocketAddr;
use std::process;
use std::time::Duration;

use getopts::Options;

pub struct Args {
    pub address: SocketAddr,
    pub cache_enable: bool,
    pub cache_ttl: Duration,
    pub cache_max_size: u64,
}

fn opts() -> Options {
    let mut opts = Options::new();
    opts.optflag(
        "h",
        "help",
        concat!("Print the help output of ", env!("CARGO_PKG_NAME")),
    );
    opts.optopt(
        "a",
        "address",
        "Socket address (IP and port) to listen on [Default: 127.0.0.1:8080]",
        "SOCKET_ADDRESS",
    );
    opts.optflag(
        "c",
        "cache-enable",
        "Enable caching of parsed calendars [Default: false]",
    );
    opts.optopt(
        "t",
        "cache-ttl",
        "Time-to-live for cached calendars [Default: 3600]",
        "SECONDS",
    );
    opts.optopt(
        "s",
        "cache-max-size",
        "Maximum cache size in Megabytes [Default: 50]",
        "MEGABYTES",
    );
    opts
}

pub fn parse(args: Vec<String>) -> Args {
    let opts = opts();

    let matches = match opts.parse(args) {
        Ok(matches) => matches,
        Err(fail) => {
            eprintln!("{fail}");
            process::exit(1);
        }
    };

    if matches.opt_present("help") {
        println!("{}", opts.usage(&opts.short_usage(env!("CARGO_PKG_NAME"))));
        process::exit(0);
    }

    let address = match matches.opt_get_default("address", SocketAddr::from(([127, 0, 0, 1], 8080)))
    {
        Ok(address) => address,
        Err(err) => {
            eprintln!("Provided value for option 'socket-address' is invalid: {err}");
            process::exit(1);
        }
    };

    let cache_enable = matches.opt_present("cache-enable");

    let cache_ttl = match matches.opt_get_default("cache-ttl", 3600) {
        Ok(secs) => Duration::from_secs(secs),
        Err(err) => {
            eprintln!("Provided value for option 'cache-ttl' is invalid: {err}");
            process::exit(1);
        }
    };

    let cache_max_size = match matches.opt_get_default("cache-max-size", 50) {
        Ok(size) => size,
        Err(err) => {
            eprintln!("Provided value for option 'cache-max-size' is invalid: {err}");
            process::exit(1);
        }
    };

    Args {
        address,
        cache_enable,
        cache_ttl,
        cache_max_size,
    }
}
