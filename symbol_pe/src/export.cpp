#include "export.h"
#include "symbol.h"
#include "callback.hpp"
#include <iostream>
#include <dia2.h>
#include <DbgHelp.h>
#include <fstream>


size_t GetSize(const std::string& path) {
    std::fstream file(path);
    file.seekg(0, std::ios::end);
    size_t end_pos = file.tellg();
    return end_pos;
}





__declspec(dllexport) const char* GetTagString(DWORD tag) {
    switch (tag) {
    case SymTagNull: return "SymTagNull";
    case SymTagExe: return "SymTagExe";
    case SymTagCompiland: return "SymTagCompiland";
    case SymTagCompilandDetails: return "SymTagCompilandDetails";
    case SymTagCompilandEnv: return "SymTagCompilandEnv";
    case SymTagFunction: return "SymTagFunction";
    case SymTagBlock: return "SymTagBlock";
    case SymTagData: return "SymTagData";
    case SymTagAnnotation: return "SymTagAnnotation";
    case SymTagLabel: return "SymTagLabel";
    case SymTagPublicSymbol: return "SymTagPublicSymbol";
    case SymTagUDT: return "SymTagUDT";
    case SymTagEnum: return "SymTagEnum";
    case SymTagFunctionType: return "SymTagFunctionType";
    case SymTagPointerType: return "SymTagPointerType";
    case SymTagArrayType: return "SymTagArrayType";
    case SymTagBaseType: return "SymTagBaseType";
    case SymTagTypedef: return "SymTagTypedef";
    case SymTagBaseClass: return "SymTagBaseClass";
    case SymTagFriend: return "SymTagFriend";
    case SymTagFunctionArgType: return "SymTagFunctionArgType";
    case SymTagFuncDebugStart: return "SymTagFuncDebugStart";
    case SymTagFuncDebugEnd: return "SymTagFuncDebugEnd";
    case SymTagUsingNamespace: return "SymTagUsingNamespace";
    case SymTagVTableShape: return "SymTagVTableShape";
    case SymTagVTable: return "SymTagVTable";
    case SymTagCustom: return "SymTagCustom";
    case SymTagThunk: return "SymTagThunk";
    case SymTagCustomType: return "SymTagCustomType";
    case SymTagManagedType: return "SymTagManagedType";
    case SymTagDimension: return "SymTagDimension";
    default: return "Unknown";
    }
}



Symbol* getSymbols(size_t* len, const char* path) {
    try {
        SymbolPdb extractor;
        auto symbols = extractor.getSymbols(path);
        *len = symbols.size();
        Symbol* symbol_ar = new Symbol[symbols.size()];
        std::copy(symbols.begin(), symbols.end(), symbol_ar);
        return symbol_ar;
    }
    catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return nullptr;
    }
}



Symbol* GetPdbSym(const char* pdb_path, size_t* len, DWORD64 base_addr) {
    try {
        SymbolPdb extractor;
        auto symbols = extractor.getSymbolsForPdb(pdb_path, base_addr);
        *len = symbols.size();
        Symbol* symbol_ar = new Symbol[symbols.size()];
        std::copy(symbols.begin(), symbols.end(), symbol_ar);
        return symbol_ar;
    }
    catch (const std::exception& e) {
        std::cerr << "Error for load pdb sym, " << e.what() << std::endl;
        return nullptr;
    }
}



LocalSym* GetLocalVar(HANDLE h_proc, DWORD64 addr_func, const CONTEXT* ctx_proc, size_t* len) {
    try {
        SymbolPdb extractor(h_proc);
        std::vector<LocalSym> symbol = extractor.GetLocalVar(addr_func, ctx_proc);
        *len = symbol.size();
        LocalSym* sym = new LocalSym[*len];
        std::copy(symbol.begin(), symbol.end(), sym);
        return sym;
    }
    catch (const std::exception& e) {
        std::cerr << "Error for get local var, " << e.what() << std::endl;
        return nullptr;
    }
}



BOOL sym_init(HANDLE h_proc, const char* pdb_path, DWORD64 base_addr) {
    SymSetOptions(SYMOPT_CASE_INSENSITIVE | SYMOPT_DEFERRED_LOADS |SYMOPT_LOAD_LINES | SYMOPT_UNDNAME | SYMOPT_INCLUDE_32BIT_MODULES | SYMOPT_DEBUG);
    if (!SymInitialize(h_proc, nullptr, TRUE))
        return FALSE;
    
    if (pdb_path) {
        if (!SymLoadModule64(h_proc, nullptr, pdb_path, nullptr, base_addr, (DWORD)GetSize(pdb_path))) 
            return FALSE;
    }
    return TRUE;
}