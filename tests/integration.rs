use std::{
    env::{self, set_current_dir},
    fs::remove_dir_all,
};

use assert_cli::Assert;

fn run_test<F: FnOnce()>(test: F) {
    let tmp_migration_dir = std::env::temp_dir().join("migrations");
    if tmp_migration_dir.exists() {
        remove_dir_all(&tmp_migration_dir)
            .unwrap_or_else(|_| panic!("Can´t delete {:#?}", &tmp_migration_dir));
    }
    set_current_dir("tests/").expect("Can´t change workingdir to tests/");
    test();
}

#[test]
fn test_migration() {
    run_test(|| Assert::main_binary().succeeds().unwrap());
}

#[test]
fn test_migration_with_wrong_postgres_host() {
    let env: Vec<(String, String)> = env::vars()
        .map(|elem| {
            if elem.0 == "POSTGRES_HOST" {
                (String::from("POSTGRES_HOST"), String::from("lolcalhost"))
            } else {
                elem
            }
        })
        .collect();

    run_test(|| {
        Assert::main_binary()
            .with_env(&env)
            .fails()
            .stderr()
            .contains("Can`t connect to database")
            .unwrap()
    });
}

#[test]
fn test_migration_without_postgres_host() {
    let env: Vec<(String, String)> = env::vars()
        .filter(|(key, _)| key != "POSTGRES_HOST")
        .collect();

    run_test(|| {
        Assert::main_binary()
            .with_env(&env)
            .fails()
            .stderr()
            .contains("Environment variable `POSTGRES_HOST` not found")
            .unwrap()
    });
}

#[test]
fn test_migration_without_postgres_db() {
    let env: Vec<(String, String)> = env::vars()
        .filter(|(key, _)| key != "POSTGRES_DB")
        .collect();

    run_test(|| {
        Assert::main_binary()
            .with_env(&env)
            .fails()
            .stderr()
            .contains("Environment variable `POSTGRES_DB` not found")
            .unwrap()
    });
}

#[test]
fn test_migration_without_postgres_password() {
    let env: Vec<(String, String)> = env::vars()
        .filter(|(key, _)| key != "POSTGRES_PASSWORD")
        .collect();

    run_test(|| {
        Assert::main_binary()
            .with_env(&env)
            .fails()
            .stderr()
            .contains("Environment variable `POSTGRES_PASSWORD` not found")
            .unwrap()
    });
}
