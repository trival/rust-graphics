use notify::{RecursiveMode, Watcher};
use serde::Deserialize;
use std::{
	collections::BTreeMap,
	env,
	fs::File,
	io::Read,
	path::{Path, PathBuf},
	process::{Child, Command},
	sync::mpsc::channel,
};

#[derive(Debug, Deserialize)]
struct CargoToml {
	package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
	name: String,
}

struct RunningProcess {
	child: Child,
}

impl RunningProcess {
	fn new(crate_name: &str) -> std::io::Result<Self> {
		let child = Command::new("cargo")
			.args(["run", "--bin", crate_name])
			.spawn()?;
		Ok(RunningProcess { child })
	}

	fn terminate(&mut self) -> std::io::Result<()> {
		self.child.kill()
	}
}

fn read_crate_name(cargo_toml_path: &Path) -> std::io::Result<String> {
	let mut content = String::new();
	File::open(cargo_toml_path)?.read_to_string(&mut content)?;

	let cargo_toml: CargoToml = toml::from_str(&content)
		.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

	Ok(cargo_toml.package.name)
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		eprintln!("Usage: {} <crate-directory>", args[0]);
		return Ok(());
	}

	let crate_path = PathBuf::from(&args[1]);
	let cargo_toml_path = crate_path.join("Cargo.toml");
	let src_path = crate_path.join("src");

	let crate_name = read_crate_name(&cargo_toml_path)?;
	println!("Watching crate: {}", crate_name);

	let (tx, rx) = channel();
	let mut watcher = notify::recommended_watcher(tx).unwrap();
	watcher.watch(&src_path, RecursiveMode::Recursive).unwrap();

	let mut current_process = RunningProcess::new(&crate_name)?;

	let mut event_table = BTreeMap::new();

	for res in rx {
		match res {
			Ok(e) => {
				if e.kind.is_modify() {
					let current_time = std::time::SystemTime::now();
					if let Some(last_event_time) = event_table.get(&e.paths) {
						if current_time
							.duration_since(*last_event_time)
							.unwrap()
							.as_millis()
							< 500
						{
							continue;
						}
					}

					println!("Rebuilding...");
					let status = Command::new("cargo")
						.args(["build", "--bin", &crate_name])
						.status()?;

					if status.success() {
						println!("Build successful, restarting process");
						current_process.terminate()?;
						current_process = RunningProcess::new(&crate_name)?;
					} else {
						println!("Build failed");
					}

					let current_time = std::time::SystemTime::now();
					event_table.insert(e.paths, current_time);
				}
			}

			Err(e) => println!("Watch error: {:?}", e),
		}
	}

	Ok(())
}
