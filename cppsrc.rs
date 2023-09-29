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
	#[cfg(feature="_cpp")]
	{
		let debug = if std::env::var("_DEBUG") == Ok(String::from("true")) { Some( |job: &mut Build| {
			job
				.flag_if_supported("-MDd")
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
		use std::{fs, path::Path};
		let out_path = std::env::var_os("OUT_DIR").unwrap();
		let out_path = Path::new(&out_path).join("include");
		let mut dir = Option::<()>::None;
		for file in fs::read_dir("include").unwrap() {
			let file = file.unwrap();
			dir.get_or_insert_with(|| fs::create_dir(&out_path).unwrap());
			let out_path = out_path.join(file.file_name());
			fs::copy(file.path(), out_path.clone()).unwrap();
			println!("## target dir: {}", out_path.display());
			println!("cargo:rerun-if-changed={}", file.path().display());
		}
		let out_path = std::env::var_os("OUT_DIR").unwrap();
		let out_path = Path::new(&out_path).join("include");
		let mut dir = Option::<()>::None;
		for file in fs::read_dir("include").unwrap() {
			let file = file.unwrap();
			dir.get_or_insert_with(|| fs::create_dir(&out_path).unwrap());
			let out_path = out_path.join(file.file_name());
			fs::copy(file.path(), out_path.clone()).unwrap();
			println!("## target dir: {}", out_path.display());
			println!("cargo:rerun-if-changed={}", file.path().display());
		}
	}
}