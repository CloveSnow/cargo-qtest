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
pub enum Qtest {
    RunQtest {
        test_module: Vec<String>,
        targets: Option<String>,
        args: Option<Vec<String>>,
    },
    QueryTest {
        pattern: Option<String>,
    },
    QueryTestModule,
    All,
    #[default]
    NoOp,
}

impl Qtest {
    const ID_TEST: &'static str = "test";
    const ID_TARGET: &'static str = "targets";
    const ID_LIST: &'static str = "list";
    const ID_ALL: &'static str = "all";
    const ID_ARGS: &'static str = "args";

    pub fn new() -> clap::Command {
        clap::Command::new("cargo-qtest")
            .about("A Hack for prototyping and Seperate Multiple test into TEST MODULE")
            .version("1.0.0")
            .display_name("qtest")
            .arg_required_else_help(true)
            // .group(
            //     ArgGroup::new("RUN_TEST_OPT")
            //         .args([Self::ID_TARGET, Self::ID_ARGS])
            //         .requires(Self::ID_TEST)
            //         .required(false)
            //         .multiple(true),
            // )
            .arg(
                Arg::new(Self::ID_TEST)
                    .value_name("TESTS MODULE")
                    .help("The <TEST MODULE> to run")
                    .required(false)
                    .num_args(1..)
                    .action(ArgAction::Append),
            )
            .arg(
                Arg::new(Self::ID_TARGET)
                    .value_name("TARGETS")
                    .short('t')
                    .long(Self::ID_TARGET)
                    .help("Run the targets test within <TEST MODULE>")
                    .num_args(1..)
                    .requires(Self::ID_TEST)
                    .required(false)
                    .action(ArgAction::Set),
            )
            .arg(
                Arg::new(Self::ID_LIST)
                    .short('l')
                    .long("list")
                    .help("List all the test")
                    .exclusive(true)
                    .required(false)
                    .action(ArgAction::Set),
            )
            .arg(
                Arg::new(Self::ID_ALL)
                    .short('a')
                    .long(Self::ID_ALL)
                    .help("Run all tests")
                    .exclusive(true)
                    .required(false)
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new(Self::ID_ARGS)
                    .value_name("ARGS")
                    .help("Pass flags and arguments to the test binary")
                    .long_help(DESCPASSARG)
                    .allow_hyphen_values(true)
                    .action(ArgAction::Append)
                    .required(false)
                    .requires(Self::ID_TEST)
                    .last(true),
            )
    }
}
const DESCPASSARG: &'static str = r#"Pass flags and arguments to the test binary
Example
```rust
#[derive(qtest)]
fn module() {
    let result: Vec<_> = std::env::args().collect();
    let expected_result = 
        vec![String::from("foo"), String::from("--bar"), String::from("quax")];
        
    assert_eq!(result,expected_result);
}
```
``sh
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
impl ArgMatchExt for Vec<String> {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self {
        matches
            .remove_many(id)
            .map(|v| v.into_iter().collect())
            .unwrap()
    }
}
impl ArgMatches for bool {
    fn extract(matches: &mut ArgMatches, id: &str) -> Self {
        matches
            .get_flag(id)
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

impl Qtest {
    fn from_args(mut args: ArgMatches) -> Self {
        let mut args = ArgMatches1(args);
        if args.extract(Self::ID_ALL) {
            return Self::All;
        }

        if args.contains_id(Self::ID_TEST) {
            let test_module = args.extract(Self::ID_TEST);

            let targets = args.extract(Self::ID_TARGET);
            let args = args.extract(Self::ID_ARGS);

            return Self::RunQtest {
                test_module,
                targets,
                args,
            };
        };
        if args.contains_id(Self::ID_LIST) {
            return Self::QueryTest {
                pattern: args.extract(Self::ID_LIST),
            };
        };
        if args.extract(Self::ID_ALL) {
            return Self::All;
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

// fn produce_test<const N: usize, const M: usize>(
//     will_pass: bool,
//     args: [&str; N],
//     expected_result: [(&str, &str); M],
// ) {
//     let matches = Qtest::new().get_matches_from(args);

//     for (id, expected)

//     let result = match (Qtest::from_arg_matches(&matches), will_pass) {
//         (Ok(result), true) => result,
//         (Err(_), false) => return,
//         (Ok(_), false) => panic!("The arguments was expected to fail to be parsed"),
//         (Err(_), true) => panic!("The arguments was expected to successfully parsed"),
//     };
//     assert_eq!(result, expected_result,);
// }
#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! test {
        ([$($args: literal),*$(,)?], $($id: path [ $type: ty]  => $expected_value: expr ),+$(,)?) => {

            let mut matches = Qtest::new().try_get_matches_from([$($args,)*]).unwrap();
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
    type Target = Option<String>;
    type Args = Option<Vec<String>>;

    test!(@PASS, pass,
        {
            @ARGS: ["cargo-qtest", "module"],
            Qtest::ID_TEST[Tests] => strings!("module"),
            Qtest::ID_TARGET[Target] => None,
            Qtest::ID_ARGS[Args] => None,
        },
        {
            @ARGS: ["cargo-qtest", "module", "module2"],
            Qtest::ID_TEST[Tests] => strings!("module", "module2"),
            Qtest::ID_TARGET[Target] => None,
            Qtest::ID_ARGS[Args] => None,
        },
        {
            @ARGS: ["cargo-qtest", "module", "-t", "foo"],
            Qtest::ID_TEST[Tests] => strings!("module"),
            Qtest::ID_TARGET[Target] => Some(string!("foo")),
            Qtest::ID_ARGS[Args] => None,
        },
    );
    test!(@FAIL, fail,
        {
            @ARGS: ["cargo-qtest", "module", "-t" ],
            Qtest::ID_TEST[Tests] => strings!("module", "module2"),
            Qtest::ID_ARGS[Args] => None,
            Qtest::ID_TARGET[Target] => None,
        },
    );

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
