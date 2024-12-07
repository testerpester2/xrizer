use nanoserde::DeJson;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

// https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages
#[derive(DeJson)]
struct Artifact {
    target: ArtifactTarget,
    filenames: Vec<String>,
}

#[derive(DeJson)]
struct ArtifactTarget {
    name: String,
    crate_types: Vec<String>,
}

enum Message {
    CompilerArtifact(Artifact),
    Unknown,
}

fn main() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo)
        .args([
            "build",
            "--message-format",
            "json-render-diagnostics",
            "-p",
            "xrizer",
        ])
        .args(std::env::args_os().skip(1))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to call cargo");

    let stdout = cmd.stdout.take().unwrap();
    let mut stdout = BufReader::new(stdout);

    let mut lib_path: Option<String> = None;
    let mut line = String::new();

    while stdout.read_line(&mut line).expect("Failed to read line") > 0 {
        let msg = Message::deserialize_json(&line).unwrap();
        line.clear();

        match msg {
            Message::CompilerArtifact(a) => {
                let target = a.target;
                if !(target.name == "xrizer" && target.crate_types.contains(&"cdylib".into())) {
                    continue;
                }

                lib_path = Some(
                    a.filenames
                        .into_iter()
                        .find(|p| p.ends_with(".so"))
                        .unwrap(),
                )
            }
            Message::Unknown => {}
        }
    }

    if !cmd.wait().expect("waiting for build failed").success() {
        std::process::exit(1);
    }
    let lib_path = PathBuf::from(lib_path.expect("lib path missing"));

    let parent = lib_path.parent().unwrap();
    match std::fs::create_dir_all(parent.join("bin/linux64")) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        err => {
            eprintln!("Failed to create bin/linux64 directory: {err:?}");
            std::process::exit(1);
        }
    }

    let vrclient_path = parent.join("bin/linux64/vrclient.so");
    match std::os::unix::fs::symlink(lib_path, vrclient_path) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        err => {
            eprintln!("Failed to create vrclient symlink: {err:?}");
            std::process::exit(1);
        }
    }
}

impl DeJson for Message {
    fn de_json(
        state: &mut nanoserde::DeJsonState,
        input: &mut std::str::Chars,
    ) -> Result<Self, nanoserde::DeJsonErr> {
        state.curly_open(input)?;
        let key = String::de_json(state, input)?;
        if key != "reason" {
            return Ok(Self::Unknown);
        }
        state.colon(input)?;
        let reason = String::de_json(state, input)?;
        match reason.as_str() {
            "compiler-artifact" => {
                let fixed: String = ['{', state.cur].into_iter().chain(input).collect();
                let msg = Artifact::deserialize_json(&fixed).unwrap();
                Ok(Self::CompilerArtifact(msg))
            }
            _ => Ok(Self::Unknown),
        }
    }
}
