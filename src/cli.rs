#![allow(unused_imports)]
use clap::{
    builder::ValueParser, Arg, ArgAction, ArgGroup, ArgMatches, Args, CommandFactory,
    FromArgMatches, Parser,
};
use std::default;
use std::ffi::OsString;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Default)]
pub enum Qtest {
    RunQtest {
        test_module: String,
        targets: Option<Vec<String>>,
        args: Option<Vec<String>>,
    },
    QueryTest {
        test_module: String,
    },
    QueryTestModule,
    Clean,
    All,
    #[default]
    NoOp,
}

impl Qtest {
    pub fn new() -> clap::Command {
        clap::Command::new("cargo-qtest")
            .about("A Hack for prototyping and Seperate Multiple test into TEST MODULE")
            .version("1.0.0")
            .display_name("qtest")
            .arg_required_else_help(true)
            .group(
                ArgGroup::new("RUN")
                    .args(["test_module", "list_test", "args"])
                    .multiple(true),
            )
            .arg(
                Arg::new("test_module")
                    .value_name("TEST MODULE")
                    .help("The TEST MODULE to run")
                    .value_parser(ValueParser::string())
                    .action(ArgAction::Set)
                    .group("TEST MODULE"), // .index(1),
            )
            .arg(
                Arg::new("targets")
                    .value_name("TARGET")
                    .short('t')
                    .long("targets")
                    .help("Run the targets test within <TEST MODULE>")
                    .requires("test_module")
                    .num_args(1..)
                    .value_parser(ValueParser::string())
                    .action(ArgAction::Set),
            )
            .arg(
                Arg::new("list_module")
                    .short('m')
                    .long("list-module")
                    .help("List all the TEST MODULE")
                    .exclusive(true)
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("list_test")
                    .short('f')
                    .long("list-test")
                    .help("List all the test within the <TEST MODULE>")
                    .requires("test_module")
                    .conflicts_with("list_module")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("all")
                    .short('a')
                    .long("all")
                    .help("Run all tests")
                    .exclusive(true)
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("clean")
                    .short('c')
                    .long("clean")
                    .help("Clean qtest from manifest")
                    .exclusive(true)
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("args")
                    .value_name("ARGS...")
                    .help("Pass flags and arguments to the test binary")
                    .long_help(DESCPASSARG)
                    .requires("TEST MODULE")
                    .allow_hyphen_values(true)
                    .last(true),
            )
    }
}
const DESCPASSARG: &'static str = r#"Pass flags and arguments to the test binary
Example
```rust
#[derive(qtest)]
fn module() {
    use std::convert::Infallible;
    use std::ffi::OsString;
    let expected_result: Vec<OsString> = ["foo", "--bar", "quax"]
        .into_iter()
        .map(OsString::from_str)
        .map(<Result<OsString, Infallible>>::unwrap)
        .collect();

    let result: Vec<OsString> = std::env::args_os()
        .into_iter()
        .filter(|arg| expected_result.contains(arg))
        .collect();

    assert_eq!(&expected_result[0..3], &result[0..3]);
}
```
```sh
    $ cargo qtest module -- foo --bar quax
```
"#;
impl CommandFactory for Qtest {
    fn command() -> clap::Command {
        Self::new()
    }

    fn command_for_update() -> clap::Command {
        Self::new()
    }
}

impl Qtest {
    fn from_args(mut args: ArgMatches) -> Self {
        if args.contains_id("test_module") {
            let test_module = args.remove_one("test_module").unwrap();

            if args.get_flag("list_test") {
                return Self::QueryTest { test_module };
            }

            let targets = match args.remove_many::<String>("targets") {
                None => None,
                Some(targets) => Some(targets.map(|target| target).collect()),
            };

            let args = match args.remove_many::<String>("args") {
                None => None,
                Some(args) => Some(args.map(|arg| arg).collect()),
            };

            return Self::RunQtest {
                test_module,
                targets,
                args,
            };
        };
        if args.get_flag("all") {
            return Self::All;
        }
        if args.get_flag("list_module") {
            return Self::QueryTestModule;
        }
        if args.get_flag("clean") {
            return Self::Clean;
        }
        return Self::NoOp;
    }
}

impl FromArgMatches for Qtest {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let args = matches.clone();
        Ok(Self::from_args(args))
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        let args = matches.clone();
        *self = Self::from_args(args);
        Ok(())
    }
}

impl Parser for Qtest {}

#[cfg(test)]
mod test_qtest {

    use clap::{FromArgMatches, Parser};

    use super::Qtest;

    fn produce_test<const N: usize>(
        will_pass: bool,
        args: [&'static str; N],
        expected_result: Qtest,
    ) {
        let matches = Qtest::new().get_matches_from(args);
        let result = match (Qtest::from_arg_matches(&matches), will_pass) {
            (Ok(result), true) => result,
            (Err(_), false) => return,
            (Ok(_), false) => panic!("The arguments was expected to fail to be parsed"),
            (Err(_), true) => panic!("The arguments was expected to successfully parsed"),
        };
        assert_eq!(result, expected_result,);
    }
    #[test]
    fn cli_qtest() {
        produce_test(
            true,
            ["module"],
            Qtest::RunQtest {
                test_module: "module".into(),
                targets: None,
                args: None,
            },
        );
        produce_test(
            true,
            ["module", "-f"],
            Qtest::QueryTest {
                test_module: "module".into(),
            },
        );
        produce_test(
            true,
            ["--list-module"],
            Qtest::QueryTest {
                test_module: "module".into(),
            },
        );
        produce_test(
            true,
            ["module", "--list-test"],
            Qtest::QueryTest {
                test_module: "module".into(),
            },
        );
        produce_test(true, ["-a"], Qtest::All);
        produce_test(true, ["--all"], Qtest::All);
        produce_test(false, ["--all -l"], Default::default());
        produce_test(false, ["module", "--list-module"], Default::default());
        produce_test(false, ["--list-test"], Default::default());
    }
}

// Reference
// https://manpages.org/rustc
// Enviroment flags we must remove
struct RustcFlags {
    // #[arg(short = 'A', long)]
    allow: Option<Vec<String>>,
    // #[arg(short = 'D', long)]
    deny: Option<Vec<String>>,
    // #[arg(short = 'F', long)]
    forbid: Option<Vec<String>>,
    // #[arg(short, long)]
    help: bool,
    // #[arg(long)]
    cfg: Option<Vec<String>>,
    // #[arg(short)]
    library: Vec<(LibaryFlags, std::path::PathBuf)>,
}

#[allow(dead_code)]
enum LibaryFlags {
    Dependency,
    Crate,
    Native,
    FrameWork,
    All,
}

#[test]
#[allow(unused_variables, unreachable_code)]
#[ignore = "Todo Parse the global environment of RUSTFLAG to only include necssary flags"]
fn rustcflags() {
    todo!("PARSE the global environment for RUSTFLAGS");
    let rustcflags: OsString = OsString::from_str("-A HELLO -D HELLO -F HELLO").unwrap();
}
