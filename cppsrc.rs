#![feature(path_file_prefix)]

use std::path::PathBuf;
use cc::Build;

#[allow(dead_code)]
fn compile<F : FnOnce(&mut Build)> (file: PathBuf, process: Option<F>) -> Result<cc::Build, PathBuf> {
	if file.extension() == Some("cpp".as_ref()) {
		let Some(name) = file.file_prefix() else { return Err(file) };
		let mut job = Build::new();
		job
			.cpp(true)
			.std("c++14")
			.file(file.clone())
	    	.include("include");
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
	#[cfg(feature="_cpp")]
	{
		let debug = if std::env::var("DEBUG") == Ok(String::from("true")) { Some( |job: &mut Build| {
			if job.get_compiler().is_like_msvc() {
				job.flag("-MDd");
			}
			job
					.flag_if_supported("-fms-runtime-lib=dll_dbg")
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
}
