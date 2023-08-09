fn main() {
	cc::Build::new()
	    .cpp(true)
	    .include("include")
		.file("src/rs8080.cpp")
		.compile("rs8080");
}