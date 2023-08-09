fn main() {
	cc::Build::new()
	    .cpp(true)
		.file("src/panic.cpp")
		.compile("panic");
}