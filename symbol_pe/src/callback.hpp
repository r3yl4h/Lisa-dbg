#include <Windows.h>
#include <DbgHelp.h>


struct SymCtxLocal {
    HANDLE process;
    std::vector<LocalSym>* symbolVector;
    const CONTEXT* ctx_proc;
};

struct SymCtx {
    HANDLE process;
    std::vector<Symbol>* symbolVector;
};



BOOL CALLBACK SymCall(PSYMBOL_INFO, ULONG, PVOID);
BOOL CALLBACK CallBack4LocalVar(PSYMBOL_INFO, ULONG, PVOID);