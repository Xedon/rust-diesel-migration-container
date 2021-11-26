use std::{
    fs::{create_dir_all, File},
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};

use diesel::{Connection, ConnectionError};
use diesel_migrations::run_pending_migrations_in_directory;
use globwalk::glob;
use tera::Context;

type ConnectionType = diesel::pg::PgConnection;

const MIGRATION_DIR_GLOB: &str = "migrations/**/*.sql";
const MAX_CONNECTION_TRYS: u8 = 60;

fn main() {
    let term = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))
        .unwrap_or_else(|err| panic!("Unable to register SIGTERM hook {:?}", err));

    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))
        .unwrap_or_else(|err| panic!("Unable to register SIGINT hook {:?}", err));

    let mut renderer = tera::Tera::new(MIGRATION_DIR_GLOB)
        .unwrap_or_else(|err| panic!("Unable to render sql templates {:?}", err));

    let migrations_files = glob(MIGRATION_DIR_GLOB)
        .unwrap_or_else(|err| panic!("Unable to iterate over templates {:?}", err));

    let context = Context::default();

    for entry in migrations_files.into_iter() {
        match entry {
            Ok(x) => {
                if x.file_type().is_file() {
                    let target_file = std::env::temp_dir().join(x.path());
                    let target_file_parent_dir = target_file.parent().unwrap();

                    println!("Render: {:?} to {:?}", &x.path(), &target_file);
                    if !target_file_parent_dir.exists() {
                        create_dir_all(&target_file_parent_dir)
                            .unwrap_or_else(|err| panic!("Unable to create dir {:?}", err));
                    }

                    let file = File::create(target_file)
                        .unwrap_or_else(|err| panic!("Unable to create file {:?}", err));

                    renderer
                        .render_to(
                            x.path()
                                .to_str()
                                .unwrap()
                                .strip_prefix("./migrations/")
                                .unwrap(),
                            &context,
                            file,
                        )
                        .unwrap_or_else(|err| panic!("Unable render template {:?}", err));
                }
            }
            Err(err) => {
                panic!("Unable to iterate over templates {:?}", err)
            }
        }
    }

    let connection_string = renderer.render_str(
        r#"postgresql://{{ get_env(name="POSTGRES_USER",default="postgres") }}:{{ get_env(name="POSTGRES_PASSWORD") }}@{{ get_env(name="POSTGRES_HOST") }}:{{ get_env(name="POSTGRES_PORT",default="5432") }}/{{ get_env(name="POSTGRES_DB") }}"#,
        &context,
    ).unwrap_or_else(|err| panic!("Error on migration {:?}", err));

    let mut counter = 0;

    let mut connection: Option<ConnectionType> = Option::None;

    loop {
        if term.load(Ordering::Relaxed) {
            println!("Aborting!");
            exit(0);
        }

        counter += 1;

        println!(
            "Try to connect, attempt {} from {}",
            counter, MAX_CONNECTION_TRYS
        );

        let connection_result: Result<ConnectionType, ConnectionError> =
            Connection::establish(&connection_string);

        if let Ok(establisched_connection) = connection_result {
            connection = Some(establisched_connection);
            println!("Connected");
            break;
        }

        if counter == MAX_CONNECTION_TRYS {
            if let Err(err) = connection_result {
                panic!("Can`t connect to database {:?}", err);
            }
        }

        sleep(Duration::from_secs(1));
    }

    let connection: ConnectionType = connection.unwrap();

    run_pending_migrations_in_directory(
        &connection,
        std::env::temp_dir()
            .join("migrations")
            .canonicalize()
            .unwrap()
            .as_path(),
        &mut std::io::stdout(),
    )
    .unwrap_or_else(|err| panic!("Error on migration {:?}", err));
}
