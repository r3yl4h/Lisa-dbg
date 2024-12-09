#include "symbol.h"
#include <dbghelp.h>
#include <stdexcept>
#include <iostream>
#include <memory>
#include <string>
#include <fstream>
#include <psapi.h>
#include "export.h"
#include "callback.hpp"



SymbolPdb::SymbolPdb() : hproc(GetCurrentProcess()), close_handle(true) {
    if (!SymInitialize(hproc, nullptr, FALSE)) 
        throw std::runtime_error("SymInitialize error: " + std::to_string(GetLastError()));
}

SymbolPdb::SymbolPdb(HANDLE h_proc) : hproc(h_proc), close_handle(false) {
    SymSetOptions(SYMOPT_CASE_INSENSITIVE | SYMOPT_DEFERRED_LOADS |SYMOPT_LOAD_LINES | SYMOPT_UNDNAME | SYMOPT_INCLUDE_32BIT_MODULES | SYMOPT_DEBUG);
}


SymbolPdb::~SymbolPdb() {
    SymCleanup(hproc);
    if (close_handle)
        CloseHandle(hproc);
}



std::vector<LocalSym> SymbolPdb::GetLocalVar(DWORD64 addr_func, const CONTEXT* ctx_proc) {
    std::vector<LocalSym> symbols;
    IMAGEHLP_STACK_FRAME st_frame = { 0 };
    st_frame.InstructionOffset = addr_func;
    if (!SymSetContext(hproc, &st_frame, nullptr)) {
        throw std::runtime_error("SymSetContext Error : " + std::to_string(GetLastError()));
    }
    SymCtxLocal ctx = { hproc, &symbols, ctx_proc };
    if (!SymEnumSymbols(hproc, 0, "*", CallBack4LocalVar, &ctx)) 
        throw std::runtime_error("SymEnumSymbols error : " + std::to_string(GetLastError()));
   
    return symbols;
}



std::vector<Symbol> SymbolPdb::getSymbols(const char* path) {
    std::vector<Symbol> symbols;

    DWORD64 base_addr = SymLoadModule64(hproc, nullptr, path, nullptr, 0, 0);
    if (!base_addr) 
        throw std::runtime_error("SymLoadModule64 error: " + std::to_string(GetLastError()));
    
    IMAGEHLP_MODULE64 mod_info = { 0 };
    mod_info.SizeOfStruct = sizeof(IMAGEHLP_MODULE64);
    if (!SymGetModuleInfo(hproc, base_addr, &mod_info)) {
        SymUnloadModule64(hproc, base_addr);
        throw std::runtime_error("SymGetModuleInfo error: " + std::to_string(GetLastError()));
    }

    SymCtx ctx = { hproc, &symbols };

    if (!SymEnumSymbols(hproc, base_addr, nullptr, SymCall, &ctx)) {
        SymUnloadModule64(hproc, base_addr);
        throw std::runtime_error("SymEnumSymbols error: " + std::to_string(GetLastError()));
    }
    SymUnloadModule64(hproc, base_addr);
    return symbols;
}


std::vector<Symbol> SymbolPdb::getSymbolsForPdb(const char* pdb_file, DWORD64 l_addr) {
    std::vector<Symbol> sym;
    auto base = SymLoadModule64(hproc, nullptr, pdb_file, nullptr, l_addr, GetSize(pdb_file));
    if (!base)
        throw std::runtime_error("failed to load pdb file : " + std::to_string(GetLastError()));

    SymCtx ctx = { hproc, &sym };
    if (!SymEnumSymbols(hproc, base, nullptr, CallBack4LocalVar, &ctx)) {
        SymUnloadModule64(hproc, base);
        throw std::runtime_error("failed to unload module : " + std::to_string(GetLastError()));
    }

    SymUnloadModule64(hproc, base);
    return sym;
}