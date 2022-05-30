#![feature(path_try_exists, let_chains, split_as_slice)]

use clap::Parser;
use futures_util::{AsyncWriteExt, StreamExt};
use shiplift::{
    builder::RmContainerOptionsBuilder,
    tty::{Multiplexer, TtyChunk},
    ContainerOptions, Docker,
};
use std::{path::PathBuf, time::Duration};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    lang: String,
    recipe: PathBuf,
    directory: PathBuf,
    entrypoint: Option<String>,
}

#[recorder::record]
struct LangMetadata {
    default_entrypoints: Vec<String>,
}

#[recorder::record]
struct Recipe {
    expects: Vec<Expects>,
}

#[recorder::record]
struct Expects {
    err: Option<String>,
    input: Option<String>,
    output: Option<String>,
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    let docker = Docker::new();

    let recipe: Recipe = toml::from_str(
        tokio::fs::read_to_string(&args.recipe)
            .await
            .expect("Failed to read recipe")
            .as_str(),
    )
    .expect("Could not parse recipe");

    let meta: LangMetadata = toml::from_str(
        tokio::fs::read_to_string(format!("./metadata/{}.toml", args.lang))
            .await
            .expect("Failed to read language metadata")
            .as_str(),
    )
    .expect("Could not parse metadata");

    let dir_path = args
        .directory
        .as_path()
        .canonicalize()
        .expect("Could not canonicalize path");

    dir_path
        .try_exists()
        .expect("You need to specify an existing folder path");

    if args.entrypoint.is_none() {
        for entrypoint in meta.default_entrypoints {
            let entrypoint_path = dir_path.join(&entrypoint);
            if entrypoint_path.exists() {
                args.entrypoint = Some(entrypoint);
                break;
            }
        }
    }

    args.entrypoint
        .as_ref()
        .expect("You need to specify an entrypoint, since none matched the default");

    match docker
        .containers()
        .create(
            &ContainerOptions::builder(&format!("cheese-grader/runner-{}:latest", args.lang))
                .attach_stdin(true)
                .attach_stdout(true)
                .attach_stderr(true)
                .volumes(vec![format!(
                    "{}:/usr/src/code:ro",
                    dir_path.to_string_lossy()
                )
                .as_str()])
                .env(vec![format!("ENTRYPOINT={}", args.entrypoint.unwrap())])
                .build(),
        )
        .await
    {
        Ok(info) => {
            println!("{}", &info.id);
            let container = docker.containers().get(&info.id);

            container
                .start()
                .await
                .expect("Failed to start Docker container");

            let mut total_cases_passed = 0;
            let mut case_num = 0;
            for case in &recipe.expects {
                case_num += 1;
                println!("===== RUNNING CASE {} =====", case_num);

                let mut mux = container
                    .attach()
                    .await
                    .expect("Could not attach to Docker container");

                if let Some(input) = &case.input {
                    write_tty(&mut mux, input.as_bytes())
                        .await
                        .expect("Could not write to stdin");
                }

                let mut stdout = Vec::new();
                let mut stderr = Vec::new();

                while let Some(Ok(chunk)) = mux.next().await {
                    print_chunk(&chunk);

                    match chunk {
                        TtyChunk::StdOut(bytes) => {
                            stdout
                                .write_all(&bytes)
                                .await
                                .expect("Couldn't write to saved stdout");
                        }
                        TtyChunk::StdErr(bytes) => {
                            stderr
                                .write_all(&bytes)
                                .await
                                .expect("Couldn't write to saved stderr");
                        }
                        TtyChunk::StdIn(_) => unreachable!(),
                    }
                }

                println!("\n===== RESULTS FOR CASE {} =====", case_num);

                let mut pass = true;

                if case.output.is_some() && stdout != case.output.as_ref().unwrap().as_bytes() {
                    pass = false;
                    println!("Expected output: {}", case.output.as_ref().unwrap());
                    println!(
                        "Actual output: {}",
                        std::str::from_utf8(stdout.as_slice()).unwrap()
                    );
                }

                if case.err.is_some() && stderr != case.err.as_ref().unwrap().as_bytes() {
                    pass = false;
                    println!("Expected stderr: {}", case.err.as_ref().unwrap());
                    println!(
                        "Actual stderr: {}",
                        std::str::from_utf8(stderr.as_slice()).unwrap()
                    );
                }

                if pass {
                    total_cases_passed += 1;
                    println!("PASS");
                } else {
                    println!("FAIL");
                }

                // TODO - reset code directory

                container
                    .restart(Some(Duration::from_secs(10)))
                    .await
                    .expect("Failed to stop Docker container");
            }

            container
                .remove(RmContainerOptionsBuilder::default().force(true).build())
                .await
                .expect("Failed to remove Docker container");

            println!("\n===== SUMMARY =====");
            println!(
                "{}/{} cases passed",
                total_cases_passed,
                recipe.expects.len()
            );
        }
        Err(e) => eprintln!("Error creating container: {}", e),
    }
}

async fn write_tty(mux: &mut Multiplexer<'_>, bytes: &[u8]) -> Result<(), std::io::Error> {
    print!(
        "{}",
        ansi_term::Colour::Blue.paint(std::str::from_utf8(bytes).unwrap())
    );
    mux.write_all(bytes).await
}

fn print_chunk(chunk: &TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => print!("{}", std::str::from_utf8(bytes).unwrap()),
        TtyChunk::StdErr(bytes) => print!(
            "{}",
            ansi_term::Colour::Red.paint(std::str::from_utf8(bytes).unwrap())
        ),
        TtyChunk::StdIn(_) => unreachable!(),
    }
}
