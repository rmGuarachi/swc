use std::{fs::File, path::PathBuf, rc::Rc};

use swc_common::{chain, comments::SingleThreadedComments, Mark};
use swc_ecma_parser::{Syntax, TsConfig};
use swc_ecma_transforms_base::{feature::FeatureFlag, resolver};
use swc_ecma_transforms_compat::es2015::for_of;
use swc_ecma_transforms_module::common_js::{self, common_js};
use swc_ecma_transforms_testing::{test, test_fixture, FixtureTestConfig};
use swc_ecma_visit::Fold;

fn syntax() -> Syntax {
    Default::default()
}

fn ts_syntax() -> Syntax {
    Syntax::Typescript(TsConfig::default())
}

fn tr(
    config: common_js::Config,
    typescript: bool,
    comments: Rc<SingleThreadedComments>,
) -> impl Fold {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    let available_set = FeatureFlag::all();

    chain!(
        resolver(unresolved_mark, top_level_mark, typescript),
        common_js(unresolved_mark, config, available_set, Some(comments)),
    )
}

#[testing::fixture("tests/fixture/common/**/input.js")]
#[testing::fixture("tests/fixture/common/**/input.ts")]
#[testing::fixture("tests/fixture/common/**/input.cts")]
fn esm_to_cjs(input: PathBuf) {
    let is_ts = input
        .file_name()
        .map(|x| x.to_string_lossy())
        .map(|x| x.ends_with(".ts") || x.ends_with(".mts") || x.ends_with(".cts"))
        .unwrap_or_default();

    let dir = input.parent().unwrap().to_path_buf();

    let output = dir
        .join("output.js")
        .with_extension(if is_ts { "cts" } else { "cjs" });

    let config_path = dir.join("module.json");
    let config: common_js::Config = match File::open(config_path) {
        Ok(file) => serde_json::from_reader(file).unwrap(),
        Err(..) => Default::default(),
    };

    test_fixture(
        if is_ts { ts_syntax() } else { syntax() },
        &|tester| tr(config.clone(), is_ts, tester.comments.clone()),
        &input,
        &output,
        FixtureTestConfig {
            sourcemap: false,
            ..Default::default()
        },
    );
}

test!(
    syntax(),
    |tester| chain!(
        for_of(for_of::Config {
            assume_array: true,
            ..Default::default()
        }),
        tr(Default::default(), false, tester.comments.clone())
    ),
    for_of_as_array_for_of_import_commonjs,
    r#"
    import { array } from "foo";

    for (const elm of array) {
        console.log(elm);
    }
"#,
    r#"
    "use strict";
    Object.defineProperty(exports, "__esModule", {
        value: true
    });
    const _foo = require("foo");
    for(let _i = 0; _i < _foo.array.length; _i++){
        const elm = _foo.array[_i];
        console.log(elm);
    }
"#
);
