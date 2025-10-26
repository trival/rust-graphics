use notify::{RecursiveMode, Watcher};
use serde::Deserialize;
use std::{
	collections::BTreeMap,
	env,
	fs::File,
	io::Read,
	path::{Path, PathBuf},
	process::{Child, Command, Stdio},
	sync::mpsc::channel,
	thread,
	time::SystemTime,
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

// Check if event should be debounced (returns true if should skip)
fn should_debounce(
	event_table: &BTreeMap<Vec<PathBuf>, SystemTime>,
	paths: &Vec<PathBuf>,
	debounce_ms: u128,
) -> bool {
	if let Some(last_event_time) = event_table.get(paths) {
		if let Ok(duration) = SystemTime::now().duration_since(*last_event_time) {
			if duration.as_millis() < debounce_ms {
				return true;
			}
		}
	}
	false
}

enum WatchAction {
	BuildShader(PathBuf),
	BuildAndRestart(String),
}

// Handle file change based on action type
fn handle_file_change(
	prefix: &str,
	action: &WatchAction,
	current_process: Option<&mut RunningProcess>,
) -> std::io::Result<()> {
	match action {
		WatchAction::BuildShader(shader_path) => {
			println!("[{}] Rebuilding shaders...", prefix);
			let status = Command::new("cargo")
				.args(["gpu", "build"])
				.current_dir(shader_path)
				.stdout(Stdio::inherit())
				.stderr(Stdio::inherit())
				.status();

			match status {
				Ok(s) if s.success() => {
					println!("[{}] Build successful", prefix);
				}
				Ok(_) => {
					println!("[{}] Build failed", prefix);
				}
				Err(e) => {
					println!("[{}] Build error: {:?}", prefix, e);
				}
			}
			Ok(())
		}
		WatchAction::BuildAndRestart(crate_name) => {
			println!("[{}] Rebuilding...", prefix);
			let status = Command::new("cargo")
				.args(["build", "--bin", crate_name])
				.status()?;

			if status.success() {
				println!("[{}] Build successful, restarting process", prefix);
				if let Some(proc) = current_process {
					proc.terminate()?;
					*proc = RunningProcess::new(crate_name)?;
				}
			} else {
				println!("[{}] Build failed", prefix);
			}
			Ok(())
		}
	}
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		eprintln!("Usage: {} <sketch-name>", args[0]);
		eprintln!("Example: {} my-sketch", args[0]);
		return Ok(());
	}

	// Automatically prepend 'sketches/' to the provided path
	let sketch_name = &args[1];
	let crate_path = PathBuf::from("sketches").join(sketch_name);
	let cargo_toml_path = crate_path.join("Cargo.toml");
	let src_path = crate_path.join("src");

	let crate_name = read_crate_name(&cargo_toml_path)?;
	println!("Watching sketch: {}", crate_name);

	// Setup shader directory watcher
	let shader_path = crate_path.join("shader");
	let shader_src_path = shader_path.join("src");

	if shader_path.exists() {
		let shader_path_clone = shader_path.clone();
		thread::spawn(move || {
			let (shader_tx, shader_rx) = channel();
			let mut shader_watcher = notify::recommended_watcher(shader_tx).unwrap();
			shader_watcher
				.watch(&shader_src_path, RecursiveMode::Recursive)
				.unwrap();

			let mut shader_event_table = BTreeMap::new();
			let action = WatchAction::BuildShader(shader_path_clone);

			println!("[Shader] Watching shader directory");

			for res in shader_rx {
				match res {
					Ok(e) => {
						if e.kind.is_modify() {
							if should_debounce(&shader_event_table, &e.paths, 500) {
								continue;
							}

							let _ = handle_file_change("Shader", &action, None);

							shader_event_table.insert(e.paths, SystemTime::now());
						}
					}
					Err(e) => println!("[Shader] Watch error: {:?}", e),
				}
			}
		});
	} else {
		println!("Note: No shader directory found, skipping shader watching");
	}

	// Setup main source watcher
	let (tx, rx) = channel();
	let mut watcher = notify::recommended_watcher(tx).unwrap();
	watcher.watch(&src_path, RecursiveMode::Recursive).unwrap();

	let mut current_process = RunningProcess::new(&crate_name)?;

	let mut event_table = BTreeMap::new();

	let action = WatchAction::BuildAndRestart(crate_name.clone());

	for res in rx {
		match res {
			Ok(e) => {
				if e.kind.is_modify() {
					if should_debounce(&event_table, &e.paths, 500) {
						continue;
					}

					handle_file_change("Main", &action, Some(&mut current_process))?;

					event_table.insert(e.paths, SystemTime::now());
				}
			}

			Err(e) => println!("[Main] Watch error: {:?}", e),
		}
	}

	Ok(())
}
