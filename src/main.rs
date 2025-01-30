#![allow(unused)]
use std::{
    borrow::{Borrow, Cow},
    collections::binary_heap::Iter,
    ffi::{OsStr, OsString},
    fmt::Debug,
    io::{BufReader, Read},
    ops::AddAssign,
    path::PathBuf,
    process::Stdio,
    str::FromStr,
};

use cargo_metadata::{
    diagnostic::{self, Diagnostic, DiagnosticCode, DiagnosticLevel},
    Artifact, BuildFinished, CompilerMessage, Error, Message,
};
mod cli;
use cli::*;

fn main() {}
const RUSTFLAGS: &'static str = "RUSTFLAGS";
const REQUIREDRUSTFLAG: &'static str = "--allow warnings --forbid unexpected_cfgs";

impl Qtest {
    // iterator that search for missing test module
    fn iter(stream: BufReader<impl Read>) -> impl Iterator<Item = TestModule> {
        let metadata = cargo_metadata::Message::parse_stream(stream);

        metadata
            .into_iter()
            .filter_map(|msg| match msg {
                Err(_e) => None,
                Ok(Message::CompilerMessage(message)) => Some(message),
                Ok(_) => None,
            })
            .map(|msg| msg.message)
            .filter_map(TestModule::from_diagonstic)
    }

    // An iteractor that use stdout to scan and search for potientatial test_module
    fn scan_modules() -> impl Iterator<Item = TestModule> {
        let env: Cow<OsStr> = std::env::var_os(RUSTFLAGS).map_or(
            Cow::Borrowed(OsStr::new(REQUIREDRUSTFLAG)),
            |flag| {
                let mut flag = flag.to_owned();
                flag.push(&REQUIREDRUSTFLAG);
                Cow::Owned(flag)
            },
        );

        let child = std::process::Command::new("cargo")
            .args(["check","--quiet", "--message-format", "json-diagnostic-short", ])
            .env(RUSTFLAGS, env)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("FAILED to run instance of cargo\nPlesase ensure you have the correct cargo permission and is installed");

        let buf = std::io::BufReader::new(child.stdout.unwrap());
        Qtest::iter(buf)
    }
}

/// Each test module is a [features](https://doc.rust-lang.org/cargo/reference/features.html#the-features-section)
/// that will be added to the cargo [mainfest file](https://doc.rust-lang.org/cargo/reference/features.html#the-features-section)
impl TestModule {
    fn new(version: usize, module_name: String) -> Self {
        Self {
            version,
            module_name,
        }
    }
    // RESEARCH: rust error messages are prone to change??
    fn from_diagonstic(diagnostic: Diagnostic) -> Option<Self> {
        if diagnostic.level != DiagnosticLevel::Error {
            return None;
        }
        match diagnostic.code {
            None => return None,
            Some(ref diagnostic) => {
                if diagnostic.code.as_str() != "unexpected_cfgs" {
                    return None;
                }
            }
        };

        return diagnostic.spans.iter().find_map(|span| {
            span.text
                .iter()
                .map(|diagonistic_span_line| diagonistic_span_line.text.as_str())
                .find_map(TestModule::from_cfg)
        });
    }
}

/// Each testmodule will have a scheme of
/// INTERNALS_CARGO_QTEST_<VERSION NUMBER>_<FEATURE_NAME/MODULE_NAME>
/// This will be later used to add into mainfest file under the [feature section](https://doc.rust-lang.org/cargo/reference/features.html#the-features-section)
#[derive(Debug, PartialEq, Eq)]
struct TestModule {
    version: usize,
    module_name: String,
}

impl TestModule {
    const PREFIX: &'static str = "INTERNALS_CARGO_QTEST_";

    fn from_cfg(cfg: &str) -> Option<Self> {
        const PREFIX: &'static str = "#[cfg(feature = \"";
        const SUFFIX: &'static str = "\")]";

        let cfg = cfg.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;

        Self::from_str(cfg).map_or(None, |scheme| Some(scheme))
    }
}

impl FromStr for TestModule {
    type Err = ();

    fn from_str(prefix_version_module: &str) -> Result<Self, Self::Err> {
        const ERROR: Result<TestModule, ()> = Err(());

        if let Some(version_module) = prefix_version_module.strip_prefix(TestModule::PREFIX) {
            // retrieve version number
            let mut version_range = 0;
            for (i, c) in version_module.chars().into_iter().enumerate() {
                match (i, c) {
                    (0, c) if !c.is_ascii() => return ERROR,
                    // there should be at least a FEATURE_NAME/MODULE_NAME at the end of scheme
                    (i, c) if i >= (version_module.len() - 1) => return ERROR,
                    (i, '0'..'9') => {}
                    (i, '_') => {
                        version_range = i;
                        break;
                    }
                    _ => return ERROR,
                }
            }
            let version = usize::from_str(&version_module[..version_range]).unwrap();

            // We can safely assume that &version_module[version_range + 1] == "_"
            let module_name = String::from(&version_module[(version_range + 1)..]);

            return Ok(Self {
                version,
                module_name,
            });
        } else {
            return ERROR;
        }
    }
}

#[cfg(test)]
mod scheme_test {
    use core::panic;
    use std::{fs::OpenOptions, str::FromStr};

    use crate::TestModule;

    fn produce_scheme_test(
        should_pass: bool,
        input: &str,
        expected_version: usize,
        expected_module_name: &str,
    ) {
        let result = match (should_pass, TestModule::from_str(&input)) {
            (true, Ok(result)) => result,
            (false, Err(_)) => return,
            (true, Err(_)) => panic!("Test was expeceted to pass:\ninput: {input}"),
            (false, Ok(_)) => panic!("Test was expected to fail:\ninput: {input}"),
        };
        assert_eq!(
            result,
            TestModule {
                version: expected_version,
                module_name: String::from(expected_module_name)
            }
        );
    }
    #[test]
    fn scheme_from_str() {
        produce_scheme_test(true, "INTERNALS_CARGO_QTEST_255_HELLO", 255, "HELLO");
        produce_scheme_test(true, "INTERNALS_CARGO_QTEST_2_HELLO", 2, "HELLO");
        produce_scheme_test(true, "INTERNALS_CARGO_QTEST_255_2HELLO", 255, "2HELLO");
        produce_scheme_test(false, "INTERNALS_CARGO_QTEST_2o55_2HELLO", 255, "2HELLO");
        produce_scheme_test(false, "INTERNALSk_CARGO_QTEST_2o55_2HELLO", 255, "2HELLO");
    }
    // #[test]
    fn produce_scheme_cfg_test(
        should_pass: bool,
        input: &str,
        expected_version: usize,
        expected_module_name: &str,
    ) {
        let result = match (should_pass, TestModule::from_cfg(&input).map(|v| v)) {
            (true, None) => panic!("Test was expected to pass:\ninput: {input}"),
            (false, Some(_)) => panic!("Test was expected to fail:\ninput: {input}"),
            (true, Some(result)) => result,
            (false, None) => return,
        };
        assert_eq!(
            result,
            TestModule {
                version: expected_version,
                module_name: expected_module_name.into()
            }
        );
    }

    #[test]
    fn scheme_from_cfg() {
        produce_scheme_cfg_test(
            true,
            "#[cfg(feature = \"INTERNALS_CARGO_QTEST_0_module\")]",
            0,
            "module",
        );
        produce_scheme_cfg_test(
            true,
            "#[cfg(feature = \"INTERNALS_CARGO_QTEST_21_module\")]",
            21,
            "module",
        );
        produce_scheme_cfg_test(
            false,
            "#[cfg(feature = \"INTErRNALS_CARGO_QTEST_21_module\")]",
            0,
            "module",
        );
        produce_scheme_cfg_test(
            false,
            "#[cfg(feature =7\"INTERNALS_CARGO_QTEST_21_module\")]",
            0,
            "module",
        );
        produce_scheme_cfg_test(
            false,
            "#[cfg(feature = \"INTERNALS_CARGO_QTEST_0i_module_11\")]",
            0,
            "module_11",
        );
    }
}
