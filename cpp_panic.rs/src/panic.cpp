
#include <stdexcept>

extern "C" {
	[[noreturn]]
	int bail() {
		throw std::runtime_error{"Error in Rust library"};
	}
}