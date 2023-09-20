#include <iostream>
#include <string>
#include <algorithm>
#include <exception>

#include <rs8080.h>

struct processor_halt : public std::exception {
    const char* what() const override { return "Processor stopped by program."; }
};

bool cp_m(uint8_t (&ram)[], uint16_t addr, uint16_t de, uint8_t c) {
    using namespace std; 
    if (5 == addr) {    
        if (c == 9) {
            uint8_t* str = &ram[de + 3];  //skip the prefix bytes    
            uint8_t* end = str;
            while (*end != '$') { ++end; }
            cout << string{str, end} << endl;
        }    
        else if (c == 2) {    
            //saw this in the inspected code, never saw it called    
            cout << "print char routine called\n";    
        }    
        return true;
    }    
    else if (0 == addr)    
    {    
        throw processor_halt{};
    }   
    return false;
}

int main() {
    return 0;
}