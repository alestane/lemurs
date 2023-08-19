#include <array>

#include <rs8080.h>

#ifndef RUSTY_INVADERS
#define RUSTY_INVADERS

class invaders : public _8080::state {
public:
    const std::array<uint8_t, 7168>& get_vram() const;
}

#endif