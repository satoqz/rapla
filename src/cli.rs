use std::env;
use std::net::SocketAddr;
use std::process;

use getopts::Options;
use tokio::time::Duration;

pub struct Args {
    pub address: SocketAddr,
    pub enable_cache: bool,
    pub cache_ttl: Duration,
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
        "enable-cache",
        "Enable caching of parsed calendars [Default: false]",
    );
    opts.optopt(
        "t",
        "cache-ttl",
        "Time-to-live for cached calendars [Default: 3600]",
        "SECONDS",
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

    let enable_cache = matches.opt_present("enable-cache");

    let cache_ttl = match matches.opt_get_default("cache-ttl", 3600) {
        Ok(secs) => Duration::from_secs(secs),
        Err(err) => {
            eprintln!("Provided value for option 'cache-ttl' is invalid: {err}");
            process::exit(1);
        }
    };

    Args {
        address,
        enable_cache,
        cache_ttl,
    }
}
