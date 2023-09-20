#pragma once

#ifndef RUST_8080
#define RUST_8080

#include <cstdint>
#include <memory>
#include <vector>

namespace i8080 {
	using byte = uint8_t;
	using word = uint16_t;
	
	class state {
	public:
		word pc;
		word sp;
		byte reg[7];
		bool c, a, p, m, z;
		bool active, interrupts;

		state() = delete;
	};

	class board {
	public:
		virtual byte read(word address) const = 0;
		virtual word read_word(word address) const { return static_cast<word>(read(address)) | (read(address + 1) << 8); }
		virtual void write(word address, byte value) = 0;
		virtual void write_word(word address, word value) { write(address, value & 0xFF); write(address + 1, value >> 8); }
		virtual byte input(byte port) = 0;
		virtual void output(byte port, byte value) = 0;
		#ifdef DEBUG
		virtual const byte* did_execute(const state& chip, byte op[4]) { return nullptr; }

		#endif
	};

	class simple_board : public board {
	public:
		byte ram[0x10000];
		byte outputs[0x100];
		byte inputs[0x100];

		byte read(word address) const override { return ram[address]; }
		void write(word address, byte value) override { ram[address] = value; }
		byte input(byte port) { return inputs[port]; }
		void output(byte port, byte value) { outputs[port] = value; }
	};

	class machine {
	public:
		struct deleter { void operator() (machine* it) const; };

		using owner = std::unique_ptr < machine, deleter > ;

		static owner install(board* host = nullptr);
		const simple_board* get_default_host() const &;

		uint8_t execute() &;
	private:
		machine() = delete;
		unsigned char : 0;
	};
}

#endif
