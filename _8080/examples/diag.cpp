#include <iostream>
#include <fstream>
#include <stdexcept>

#define _8080_INTERNALS
#include <rs8080.h>

using i8080::byte;
using i8080::word;

class CP_M : public i8080::board {
public:
    CP_M(std::istream& source) 
    {
        ram[0] = 0xC3; ram[1] = 0x00; ram[2] = 0x01;
        source >> *this; 
    }
    CP_M(std::istream&& source) : CP_M{ source } {}
    template<typename InputIterator>
    CP_M(InputIterator start, InputIterator end);
    byte operator[](word index) const { return ram[index]; }
    byte& operator[](word index) { return ram[index]; }
    byte(&operator->())[0x10000] { return ram; }

    byte read(word address) const override { return ram[address]; }
    void write(word address, byte value) override { ram[address] = value; }
    byte input(byte id) override { return port[id]; }
    void output(byte id, byte value) override { port[id] = value; }

    const byte* did_execute(const i8080::state& chip, byte op[4]) override;
    
private:
    byte ram[0x10000] = { 0 };
    byte port[0x100] = { 0 };
    byte dead = 0;

    friend std::istream& operator>>(std::istream& source, CP_M& target)
    {
        return source.read(reinterpret_cast<char*>(target.ram + 0x100), 0xFF00);
    }
};

template <typename InputIterator>
CP_M::CP_M(InputIterator start, InputIterator end)
{
    ram[0] = 0xC3; ram[1] = 0x00; ram[2] = 0x01;
    for (auto target = ram + 0x100; start != end || (std::cerr << "loaded " << target - ram << " bytes.", false); ++start) {
        *target = *start;
        target += 1;
        if (target > ram + 0x10000) { target = ram + 0x100; }
    }
}

#ifdef _8080_INTERNALS
const byte* CP_M::did_execute(const i8080::state& chip, byte op[4]) {
    using std::cout;
    static union {
        byte op[4];
        uint32_t value;
    } response;
    switch (chip.pc) {
    case 0:
        cout << '\n';
        if (dead) {
            return reinterpret_cast<const byte*>( & (("Failed tests")[0]));
        }
        else {
            response.value = 0;
            response.op[1] = 0x76; // HALT
            return &response.op[0];
        }
    case 5:
        switch (chip.c) {
        case 2:
            cout << chip.e;
            break;
        case 9: 
            for (int n = 0; ; ++n) {
                if (ram[chip.de + n] == '$') {
                    cout.write(reinterpret_cast<const char*>(ram) + chip.de, n);
                    break;
                }
            }
            break;
        }
        response.value = 0;
        response.op[1] = 0xC9; // RET
        return &response.op[0];
    }
    return nullptr;
}
#endif

int main()
{
    using namespace std;

    cerr << hex << showbase;

    unique_ptr<CP_M> board{ new CP_M{ ifstream { "cpudiag.bin", ios::in | ios::binary } } };
    i8080::machine::owner sample = i8080::machine::install(board.get());
    size_t cycles = 0;
    try {
        for (;;) {
            uint8_t duration = sample->execute();
            if (!duration) { break; }
            cycles += duration;
        }
    }
    catch (std::exception& e) {
        cerr << "Stopped without completing after " << cycles << " cycles.\n";
        cerr << e.what() << '\n';
        return 1;
    }
    cout << "Completed successfully.\n";
    cout << "Total of " << cycles << " cycles executed.\n";
    return 0;
}

