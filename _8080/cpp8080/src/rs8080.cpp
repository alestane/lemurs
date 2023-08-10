#include "rs8080.h"

extern "C" {
	using i8080::state;
	using byte = i8080::state::byte;

	state* new_empty_state(size_t memory);
	state* new_state_with(size_t memory, const byte* ram);
	void discard_state(void*);
	const byte* state_outputs(const state* self);
	byte* state_inputs(state* self);
	const byte* state_ram(const state* self);

	uint8_t state_execute(state* self);
}

namespace i8080 {
 	void state::deleter::operator()(state* it) const { discard_state(it); } 

	state::owner state::create(word size, const byte* memory)
	{
		return owner{ memory ? new_state_with(size, memory) : new_empty_state(size ? size : 0x00010000) };
	}
	state::owner state::create(word size, buffer&& memory) { return create(size, memory.get()); }
	state::owner state::create(const std::vector<byte>& source) { return source.size() > 0x0000FFFF ? owner{} : create(static_cast<word>(source.size()), source.data()); }

	std::array<byte, 256>& state::ports_in() {
		return *reinterpret_cast<std::array<byte, 256>*>(state_inputs(this));
	}
	const std::array<byte, 256>& state::ports_out() const {
		return *reinterpret_cast<const std::array<byte, 256>*>(state_outputs(this));
	}

	const byte (&state::ram() const)[] {
		return *reinterpret_cast<const byte (*)[]>(state_ram(this));
	}
	
	uint8_t state::execute() {
		return state_execute(this);
	}
}
