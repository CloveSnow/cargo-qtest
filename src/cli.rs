#![allow(unused_imports)]
use clap::{
    builder::ValueParser, Arg, ArgAction, ArgGroup, ArgMatches, Args, CommandFactory,
    FromArgMatches, Parser,
};
use std::default;
use std::ffi::OsString;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Default)]
pub enum CliQtest {
    RunQtest(RunQtest),
    QueryTest(QueryTest),
    All,
    #[default]
    NoOp,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunQtest {
    pub test_module: Vec<String>,
    pub targets: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueryTest(Option<String>);

impl CliQtest {
    const ID_TEST_MODULE: &str = "test_module";
    const ID_TARGETS: &str = "targets";
    const ID_LIST_TEST: &str = "list";
    const ID_ALL: &str = "ALL";
    const ID_ARGS: &str = "ARGS";

    pub fn new() -> clap::Command {
        clap::Command::new("cargo-qtest")
            .about("A Hack for prototyping and Seperate Multiple test into TEST MODULE")
            .version("1.0.0")
            .display_name("qtest")
            .arg_required_else_help(true)
            .group(
                ArgGroup::new("RUN")
                    .args([Self::ID_TARGETS, Self::ID_ARGS])
                    .requires(Self::ID_TEST_MODULE)
                    .multiple(true),
            )
            .arg(
                Arg::new(Self::ID_TEST_MODULE)
                    .value_name("TEST MODULE")
                    .help("The TEST MODULE to run")
                    .value_parser(ValueParser::string())
                    .action(ArgAction::Append),
            )
            .arg(
                Arg::new(Self::ID_TARGETS)
                    .value_name("TARGET")
                    .short('t')
                    .long("targets")
                    .help("Run the targets test within <TEST MODULE>")
                    .num_args(1..)
                    .action(ArgAction::Append),
            )
            .arg(
                Arg::new(Self::ID_LIST_TEST)
                    .short('l')
                    .long("list-test")
                    .help("List or query all the test and modules")
                    .exclusive(true)
                    .action(ArgAction::Set),
            )
            .arg(
                Arg::new(Self::ID_ALL)
                    .short('a')
                    .long("all")
                    .help("Run all tests")
                    .exclusive(true)
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new(Self::ID_ARGS)
                    .value_name("ARGS...")
                    .help("Pass flags and arguments to the test binary")
                    .long_help(DESCPASSARG)
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

impl CliQtest {}

impl Parser for CliQtest {}
impl CommandFactory for CliQtest {
    fn command() -> clap::Command {
        Self::new()
    }

    fn command_for_update() -> clap::Command {
        Self::new()
    }
}

trait ArgMatchExt: QtestType {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self;
}
impl ArgMatchExt for Option<Vec<String>> {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self {
        matches.remove_many(id).map(|v| v.into_iter().collect())
    }
}
impl ArgMatchExt for Option<String> {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self {
        matches.remove_one(id)
    }
}
impl ArgMatchExt for String {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self {
        matches.remove_one(id).unwrap()
    }
}
impl ArgMatchExt for Vec<String> {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self {
        matches
            .remove_many(id)
            .map(|v| v.into_iter().collect())
            .unwrap()
    }
}
impl ArgMatchExt for bool {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self {
        matches.get_flag(id)
    }
}
struct ArgMatches1(ArgMatches);
trait QtestType {}
impl QtestType for String {}
impl QtestType for Option<String> {}
impl QtestType for Vec<String> {}
impl QtestType for Option<Vec<String>> {}
impl QtestType for bool {}
impl ArgMatches1 {
    fn extract<T: QtestType + ArgMatchExt>(&mut self, id: &str) -> T {
        T::extract(&mut self.0, id)
    }
}
impl Deref for ArgMatches1 {
    type Target = ArgMatches;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CliQtest {
    fn from_args(mut args: ArgMatches) -> Self {
        let mut args = ArgMatches1(args);
        if args.extract(Self::ID_ALL) {
            return Self::All;
        }

        if args.contains_id(Self::ID_TEST_MODULE) {
            let test_module = args.extract(Self::ID_TEST_MODULE);

            let targets = args.extract(Self::ID_TARGETS);
            let args = args.extract(Self::ID_ARGS);

            return Self::RunQtest(RunQtest {
                test_module,
                targets,
                args,
            });
        };
        if args.contains_id(Self::ID_LIST_TEST) {
            return Self::QueryTest(QueryTest(args.extract(Self::ID_LIST_TEST)));
        };

        if args.extract(Self::ID_ALL) {
            return Self::All;
        }
        return Self::NoOp;
    }
}

impl FromArgMatches for CliQtest {
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

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! test {
        ([$($args: literal),*$(,)?], $($id: path [ $type: ty]  => $expected_value: expr ),+$(,)?) => {

            let mut matches = CliQtest::new().try_get_matches_from([$($args,)*]).unwrap();
            let mut matches = ArgMatches1(matches);
            $(
                let field = matches.extract::<$type>($id);
                if field == $expected_value {

                }

                // eprintln!("Line {}, {}={:#?}", line!(), $id, &field);
                assert_eq!(field, $expected_value, "Test was expected to pass: {:#?}", $id);
            )+
        };
        (@PASS, $name: ident, $({@ARGS: [$($args: literal),*], $($id: path[$type: ty] => $expected_value: expr),+$(,)?},)+) => {
            #[test]
            fn $name() {
                $(
                    test!([$($args,)*], $($id[$type] => $expected_value),+);
                )+
            }
        };
        (@FAIL, $name: ident, $({@ARGS: [$($args: literal),*], $($id: path[$type: ty] => $expected_value: expr),+$(,)?},)+) => {
            #[test]
            #[should_panic]
            fn $name() {
                $(
                    test!([$($args,)*], $($id [ $type ] => $expected_value,)+);
                )*
            }
        };
    }
    macro_rules! strings {
        ($($str: literal),*) => {
            vec![$(String::from($str),)*]
        };
    }
    macro_rules! string {
        ($str: literal) => {
            String::from($str)
        };
    }
    type Tests = Vec<String>;
    type Target = Option<Vec<String>>;
    type Args = Option<Vec<String>>;
    type All = bool;
    type List = Option<String>;

    test!(@PASS, pass,
        {
            @ARGS: ["cargo-qtest", "module"],
            CliQtest::ID_TEST_MODULE[Tests] => strings!("module"),
            CliQtest::ID_TARGETS[Target] => None,
            CliQtest::ID_ARGS[Args] => None,
        },
        {
            @ARGS: ["cargo-qtest", "module", "module2"],
            CliQtest::ID_TEST_MODULE[Tests] => strings!("module", "module2"),
            CliQtest::ID_TARGETS[Target] => None,
            CliQtest::ID_ARGS[Args] => None,
        },
        {
            @ARGS: ["cargo-qtest", "module", "-t", "foo"],
            CliQtest::ID_TEST_MODULE[Tests] => strings!("module"),
            CliQtest::ID_TARGETS[Target] => Some(strings!("foo")),
            CliQtest::ID_ARGS[Args] => None,
        },
        {
            @ARGS: ["cargo-qtest", "-l", "gellfj" ],
            CliQtest::ID_LIST_TEST[List] => Some(string!("gellfj")),
        },
        {
            @ARGS: ["cargo-qtest", "-a" ],
            CliQtest::ID_ALL[All] => true,
        },
    );
    test!(@FAIL, fail,
        {
            @ARGS: ["cargo-qtest", "module", "-t" ],
            CliQtest::ID_TEST_MODULE[Tests] => strings!("module"),
            CliQtest::ID_ARGS[Args] => None,
            CliQtest::ID_TARGETS[Target] => None,
        },
        {
            @ARGS: ["cargo-qtest", "module", "module2", "-a" ],
            CliQtest::ID_TEST_MODULE[Tests] => strings!("module", "module2"),
            CliQtest::ID_ALL[All] => false,
        },
        {
            @ARGS: ["cargo-qtest", "module", "-l" ],
            CliQtest::ID_TEST_MODULE[Tests] => strings!("module"),
            CliQtest::ID_LIST_TEST[Args] => None,
        },
        {
            @ARGS: ["cargo-qtest", "-a",  "jfjjf"],
            CliQtest::ID_ALL[All] => false,
        },
    );
}

// Reference
// https://manpages.org/rustc
// Enviroment flags we must remove
pub struct RustcFlags {
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
