fn main() {
	cc::Build::new()
	    .cpp(true)
		.file("src/alloc.cpp")
		.compile("alloc");
}