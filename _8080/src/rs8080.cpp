#define _8080_INTERNALS
#include "rs8080.h"

using i8080::byte;
using i8080::word;
using i8080::board;

extern "C" {
	using i8080::machine;

	machine* create_machine(board*);
	const i8080::simple_board* request_default_impl(const machine& host);

	byte machine_execute(machine& host);
	bool machine_interrupt(machine& host, byte code);

	void discard_machine(machine* host);

	const i8080::state& machine_state(const machine& host);
}

extern "C" byte read_harness(const board& host, word address) 
{
	return host.read(address);
}

extern "C" word read_word_harness(const board& host, word address)
{
	return host.read_word(address);
}

extern "C" void write_harness(board& host, word address, byte value)
{
	host.write(address, value);
}

extern "C" void write_word_harness(board& host, word address, word value)
{
	host.write_word(address, value);
}

extern "C" byte input_harness(board& host, byte port)
{
	return host.input(port);
}

extern "C" void output_harness(board& host, byte port, byte value)
{
	host.output(port, value);
}

extern "C" const byte* did_execute_harness(board& host, const i8080::state& chip, byte op[4])
{
	return host.did_execute(chip, op);
}

namespace i8080 {
 	void machine::deleter::operator()(machine* it) const { discard_machine(it); } 

	machine::owner machine::install(board* host) 
	{
		return machine::owner{create_machine(host)};
	}

	const i8080::simple_board* machine::get_default_host() const &
	{
		return request_default_impl(*this);
	}

	byte machine::execute() & 
	{
		return machine_execute(*this);
	}

	bool machine::interrupt(byte code) & 
	{
		return machine_interrupt(*this, code);
	}

	const state& machine::operator*() const 
	{
		return machine_state(*this);
	}
}
