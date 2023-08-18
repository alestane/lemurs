#pragma once

#ifndef RUST_8080
#define RUST_8080

#include <cstdint>
#include <memory>
#include <vector>

namespace i8080 {
	class state {
	public:
		struct deleter { void operator() (state* it) const; };

		using byte = uint8_t;
		using word = uint16_t;
		using owner = std::unique_ptr < state, deleter > ;
		using buffer = std::unique_ptr<byte[]>;

		static owner create(word size = 0, const byte* source = nullptr);
		static owner create(word size, buffer&& source);
		static owner create(const std::vector<byte>& source);

		std::array<byte, 256>& ports_in();
		const std::array<byte, 256>& ports_out() const;
		const byte (&ram() const)[]; // const method returns reference to const byte array of unknown size
//		const std::array<byte, 7168>& get_vram() const;

#ifdef DEBUG
		using debugger = bool (*)(uint8_t (&)[], uint16_t, uint16_t, uint8_t);
		void add_listener(debugger op);
#endif

		uint8_t execute();
	private:
		unsigned char : 0;
	};
}

#endif
