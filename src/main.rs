#![feature(path_try_exists, let_chains, split_as_slice)]

use clap::Parser;
use futures_util::{AsyncWriteExt, StreamExt};
use shiplift::{
    tty::{Multiplexer, TtyChunk},
    ContainerOptions, Docker,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    lang: String,
    directory: PathBuf,
    entrypoint: Option<String>,
}

#[recorder::record]
struct Recipe {
    default_entrypoints: Vec<String>,
}

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    let docker = Docker::new();

    let recipe: Recipe = toml::from_str(
        tokio::fs::read_to_string(format!("./recipes/{}.toml", args.lang))
            .await
            .unwrap()
            .as_str(),
    )
    .expect("Could not parse recipe");

    let dir_path = args
        .directory
        .as_path()
        .canonicalize()
        .expect("Could not canonicalize path");

    dir_path
        .try_exists()
        .expect("You need to specify an existing folder path");

    if args.entrypoint.is_none() {
        for entrypoint in recipe.default_entrypoints {
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
                .auto_remove(true)
                .attach_stdin(true)
                .attach_stdout(true)
                .attach_stderr(true)
                .volumes(vec![format!(
                    "{}:/usr/src/code",
                    dir_path.to_string_lossy()
                )
                .as_str()])
                .env(vec![format!("ENTRYPOINT={}", args.entrypoint.unwrap())])
                .build(),
        )
        .await
    {
        Ok(info) => {
            let container = docker.containers().get(&info.id);

            let mut mux = container
                .attach()
                .await
                .expect("Could not attach to Docker container");

            container
                .start()
                .await
                .expect("Failed to start Docker container");

            write_tty(&mut mux, b"Kot\n")
                .await
                .expect("Could not write to stdin");

            let mut stdout = Vec::new();

            while let Some(Ok(chunk)) = mux.next().await {
                print_chunk(&chunk);

                if let TtyChunk::StdOut(bytes) = chunk {
                    stdout
                        .write_all(&bytes)
                        .await
                        .expect("Couldn't write to saved stdout");
                }
            }
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
