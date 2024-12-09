#ifndef EXPORTED_FUNCTIONS_H
#define EXPORTED_FUNCTIONS_H

#include <cstddef>
#include <windows.h>
#include "symbol.h"



extern "C" {
    __declspec(dllexport) const char* GetTagString(DWORD);
    __declspec(dllexport) Symbol* getSymbols(size_t*, const char*);
    __declspec(dllexport) LocalSym* GetLocalVar(HANDLE, DWORD64, const CONTEXT*, size_t*);
    __declspec(dllexport) void freeSymbols(Symbol*, size_t);
    __declspec(dllexport) void freeLocalSym(LocalSym*, size_t);
    __declspec(dllexport) Symbol* GetPdbSym(const char*, size_t*, DWORD64);
    __declspec(dllexport) BOOL sym_init(HANDLE, const char*, DWORD64);
}

size_t GetSize(const std::string&);

#endif
