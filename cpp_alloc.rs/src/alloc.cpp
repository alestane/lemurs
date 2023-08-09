#include <new>

extern "C" {
	void* cpp_allocate(size_t s) {
		return ::operator new(s, std::nothrow);
	}

	void cpp_deallocate(void* w) {
		return ::operator delete(w);
	}
}