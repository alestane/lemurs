#![feature(path_file_prefix)]

use std::path::PathBuf;
use cc::Build;

fn compile<F : FnOnce(&mut Build)> (file: PathBuf, process: Option<F>) -> Result<cc::Build, PathBuf> {
	if file.extension() == Some("cpp".as_ref()) {
		let Some(name) = file.file_prefix() else { return Err(file) };
		let mut job = Build::new();
		job
			.cpp(true)
			.file(file.clone())
	    	.include("include");
		#[cfg(feature="open")]
		job.define("_8080_INTERNALS", "open");
		if let Some(process) = process {
			process(&mut job);
		}
		job.compile(name.to_string_lossy().as_ref());
		Ok(job)
	} else {
		Err(file)
	}

}

fn main() {
	let debug = if std::env::var("DEBUG") == Ok(String::from("true")) { Some( |job: &mut Build| {
		job
			.flag("-MDd")
			.define("_ITERATOR_DEBUG_LEVEL", "2");
	})
	} else { None };
	std::fs::read_dir("src").map(
		|dir| {
			for source in dir.flatten() {
				let _ = compile(source.path(), debug);		
			}
		}
	).ok();
}