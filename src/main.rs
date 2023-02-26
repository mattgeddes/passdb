use clap::{arg, Arg, ArgMatches, Command};
use log::info;
use std::env;
use std::io;
use std::io::Write;

mod repo;
use repo::CredentialRepository;

mod creds;
use creds::Credential;

// Wrapper function to keep clap command line parsing options and arguments self-contained.
fn arg_matches() -> ArgMatches {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg(
            Arg::new("repository")
                .long("repository")
                .short('s')
                .takes_value(true)
                .help("Path to password repository"),
        )
        .subcommand(Command::new("init").about("Create repository"))
        .subcommand(Command::new("list").about("List credentials"))
        .subcommand(
            Command::new("get")
                .about("Retrieve a credential")
                .arg(arg!(<credential> "Credential to retrieve"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("set")
                .about("Store a credential")
                .arg(arg!(<credential> "Credential to store"))
                .arg_required_else_help(true),
        )
        .get_matches()
}

fn main() {
    // RUST_LOG=debug ./pwd to see log output
    env_logger::init();

    let home = match env::var_os("HOME") {
        Some(h) => h.into_string().unwrap(),
        None => panic!("$HOME isn't set"),
    };
    // Default repo path...
    let repo = home + "/.creds";
    println!("Repo directory: {}", repo);

    let args = arg_matches(); // parse command line args

    let repository = args.value_of("repository").unwrap_or(&repo).to_string();

    info!("CredentialRepository path set to '{}'", repository);

    // Create a new repository instance to act on
    let r = CredentialRepository { path: repository };

    match args.subcommand() {
        Some(("list", _)) => {
            info!("Listing credentials");
            match r.list() {
                Ok(_) => {}
                Err(e) => {
                    panic!("Unable to list repository contents: {}", e);
                }
            }
        }
        Some(("init", _)) => {
            info!("Initialising repository");
            match r.init() {
                Ok(_) => {}
                Err(e) => {
                    panic!("Failed to initialise repository: {}", e);
                }
            }
        }
        Some(("get", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("credential")
                .expect("required");
            info!("Retrieving credential: {}", name);
            match r.get(name) {
                Ok(c) => {
                    let key = rpassword::prompt_password("Enter decryption password: ").unwrap();
                    println!(
                        "Retrieved: {} -> '{}:{}'",
                        c.name,
                        c.account,
                        c.decrypt(&key).unwrap()
                    );
                }
                Err(e) => {
                    panic!("Unable to retrieve '{}': {}", name, e);
                }
            }
        }
        Some(("set", sub_matches)) => {
            let name = sub_matches
                .get_one::<String>("credential")
                .expect("required");
            info!("Setting credential: {}", name);

            let mut username = String::new();
            print!("Enter username: ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut username).unwrap();

            let mut password = String::new();
            print!("Enter password: ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut password).unwrap();

            let key = rpassword::prompt_password("Enter encryption password: ").unwrap();
            match r.set(name, username.trim_end(), password.trim_end(), &key) {
                Ok(_) => {
                    info!("Done?");
                }
                Err(e) => {
                    panic!("Unable to set '{}': {}", name, e);
                }
            }
        }
        _ => {
            println!("Error: No command given.");
        }
    }
}
