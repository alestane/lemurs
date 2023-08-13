#![feature(path_file_prefix)]

fn main() {
	std::fs::read_dir("src").map(
		|dir| {
			for source in dir.flatten() {
				let lib = source.path();
				if  lib.extension() == Some("cpp".as_ref()) {
					let Some(name) = lib.file_prefix() else { continue };
					cc::Build::new()
						.cpp(true)
						.file(lib.clone())
						.compile(name.to_string_lossy().as_ref());	
				}			
			}
		}
	).ok();
}