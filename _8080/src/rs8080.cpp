#include "rs8080.h"

extern "C" {
	i8080::state* new_empty_state(size_t memory);
	i8080::state* new_state_with(size_t memory, const i8080::state::byte* ram);
}

namespace i8080 {
	state::owner state::create(word size, const byte* memory)
	{
		return owner{ memory ? new_state_with(size, memory) : new_empty_state(size ? size : 0x00010000) };
	}
	state::owner state::create(word size, buffer&& memory) { return create(size, memory.get()); }
	state::owner state::create(const std::vector<byte>& source) { return source.size() > 0x0000FFFF ? owner{} : create(static_cast<word>(source.size()), source.data()); }
}
