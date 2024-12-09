#ifndef SYMBOL_EXTRACTOR_H
#define SYMBOL_EXTRACTOR_H

#include <vector>
#include <string>
#include <windows.h>

typedef struct Symbol {
    ULONG   Size;
    ULONG64 Value;
    ULONG64 Address;
    ULONG   Tag;
    char* Name;
    char* filename;
    DWORD   line_num;
} Symbol;



typedef struct LocalSym {
    ULONG Size;
    ULONG64 Value;
    ULONG64 Address;
    ULONG Tag;
    char* Name;
    char* filename;
    DWORD line_num;
    ULONG Reg;
} LocalSym;




class SymbolPdb {
public:
    SymbolPdb();
    SymbolPdb(HANDLE h_proc);
    ~SymbolPdb();
    std::vector<Symbol> getSymbols(const char*);
    std::vector<Symbol> getSymbolsForPdb(const char*, DWORD64);
    std::vector<LocalSym> GetLocalVar(DWORD64, const CONTEXT* ctx_proc);

private:
    HANDLE hproc;
    HANDLE hfile;
    bool close_handle;
};

#endif