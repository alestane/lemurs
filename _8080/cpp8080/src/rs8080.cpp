#include "rs8080.h"

extern "C" {
	using i8080::state;
	using byte = i8080::state::byte;

	state* entrust_blank_state(size_t memory);
	state* entrust_state_from(size_t memory, const byte* ram);
	void discard_state(void*);
	const byte* state_outputs(const state* self);
	byte* state_inputs(state* self);
	const byte* state_ram(const state* self);

#ifdef DEBUG
	void state_register_debug(state* self, state::debugger);
#endif

	uint8_t state_execute(state* self);
}

namespace i8080 {
 	void state::deleter::operator()(state* it) const { discard_state(it); } 

	state::owner state::create(word size, const byte* memory)
	{
		return owner{ memory ? entrust_state_from(size, memory) : entrust_blank_state(size ? size : 0x00010000) };
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

#ifdef DEBUG
	void state::add_listener(state::debugger listener) {
		state_add_debugger(this, listener);
	}
#endif
	
	uint8_t state::execute() {
		return state_execute(this);
	}
}
