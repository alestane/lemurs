
#include <stdexcept>

extern "C" {
	[[noreturn]]
	int bail(const char* err_text) {
		throw std::runtime_error{err_text};
	}
}