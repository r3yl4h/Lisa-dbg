#include "export.h"



void freeLocalSym(LocalSym* symbols, size_t len) {
    if (symbols == nullptr) return;
    for (size_t i = 0; i < len; ++i) {
        std::free(symbols[i].Name);
        std::free(symbols[i].filename);
    }
    delete[] symbols;
}



void freeSymbols(Symbol* symbols, size_t len) {
    if (symbols == nullptr) return;
    for (size_t i = 0; i < len; ++i) {
        std::free(symbols[i].Name);
        std::free(symbols[i].filename);
    }
    delete[] symbols;
}
