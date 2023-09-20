#include <iostream>
#include <vector>
#include <array>
#include <algorithm>

#include <rs8080.h>

using machine = i8080::state;

std::string body = "I am the very model of a model major general";

int main() {
    using namespace std;
    try {
        cout << "one two one two\n";
        auto data = vector<uint8_t>(body.begin(), body.end());
        data.resize(256, '\0');
        reverse(data.begin(), data.end());
        machine::owner chip = machine::create(data);
        for (string text : {"I've information vegetable, animal and mineral", "I know the kings of England and I quote the fights historical", "From marathon to Waterloo, in order categorical"}) {
            auto& target = chip->ports_in();
            fill(copy(text.begin(), text.end(), begin(target)), end(target), 0);
            chip->execute();
            cout << '>' << ' ' << &chip->ports_out()[0] << '\n';
        }
        cout << "done" << endl;
        return 0;
    }
    catch (exception& e) {
        cerr << "Stopped: " << e.what() << endl;
        return 17;
    }
}