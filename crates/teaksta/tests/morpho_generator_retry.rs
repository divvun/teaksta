//! A failed generator load must not stick. `TEAKSTA_GENERATOR` is process
//! global, so this lives in a test binary of its own: rewriting it here
//! cannot reach the other integration tests, and the single test in it is
//! the only thread that touches the environment.
//!
//! The first half needs no models — it proves that two different faults are
//! reported differently, which a remembered failure could not do. The second
//! half runs only when a real generator is configured.

use teaksta::morpho::{GENERATOR_ENV, MorphoPipeline};

const MISSING: &str = "/nonexistent/teaksta-generator.hfstol";
const LINE: &str = "viessu+N+Sg+Ill";

#[test]
fn a_failed_generator_load_is_retried_not_remembered() {
    let configured = std::env::var(GENERATOR_ENV).ok();
    let morpho = MorphoPipeline::shared();

    unsafe { std::env::set_var(GENERATOR_ENV, MISSING) };
    let unreadable = morpho
        .generate(LINE)
        .expect_err("generator file is missing");
    assert!(
        unreadable.to_string().contains(MISSING),
        "expected the unreadable path in: {unreadable}"
    );

    unsafe { std::env::remove_var(GENERATOR_ENV) };
    let unset = morpho
        .generate(LINE)
        .expect_err("generator is unconfigured");
    assert!(
        unset.to_string().contains(GENERATOR_ENV),
        "expected the variable name in: {unset}"
    );
    assert!(
        !unset.to_string().contains(MISSING),
        "the first failure was replayed instead of retried: {unset}"
    );

    let Some(path) = configured else {
        eprintln!("skipped the recovery half: {GENERATOR_ENV} not set");
        return;
    };
    unsafe { std::env::set_var(GENERATOR_ENV, &path) };
    let out = morpho
        .generate(LINE)
        .expect("generation recovers once the environment is repaired");
    assert!(
        out.contains(&format!("{LINE}\t")),
        "no generated form:\n{out}"
    );
}
