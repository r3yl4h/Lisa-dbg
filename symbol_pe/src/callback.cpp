#include <Windows.h>
#include <DbgHelp.h>
#include <vector>
#include "symbol.h"
#include "export.h"
#include "callback.hpp"
#include <iostream>



BOOL CALLBACK SymCall(PSYMBOL_INFO psym_info, ULONG size, PVOID Context) {
    auto* ctx = static_cast<SymCtx*>(Context);
    auto* symv = ctx->symbolVector;
    Symbol sd;
    sd.Size = psym_info->Size;
    sd.Address = psym_info->Address;
    sd.Name = _strdup(psym_info->Name);
    sd.Tag = psym_info->Tag;
    sd.Value = psym_info->Value;
    IMAGEHLP_LINE64 line = { sizeof(IMAGEHLP_LINE64) };
    DWORD disp = 0;
    if (SymGetLineFromAddr64(ctx->process, psym_info->Address, &disp, &line)) {
        sd.line_num = line.LineNumber;
        sd.filename = _strdup(line.FileName ? line.FileName : "Unknown");
    }
    else {
        sd.line_num = 0;
        sd.filename = _strdup("Unknown");
    }
    symv->push_back(sd);
    return TRUE;
}


bool IsWow64proc(HANDLE h_proc) {
    BOOL result = false;
    IsWow64Process(h_proc, &result);
    return result;
}


int64_t get_reg_value(ULONG reg, const CONTEXT* ctx_proc, bool is_wow64) {
    if (is_wow64) {
        WOW64_CONTEXT ctx = *reinterpret_cast<const WOW64_CONTEXT*>(ctx_proc);
        if (reg == 17)
            return ctx.Eax;
        if (reg == 18)
            return ctx.Ecx;
        if (reg == 19)
            return ctx.Edx;
        if (reg == 20)
            return ctx.Ebx;
        if (reg == 21)
            return ctx.Esp;
        if (reg == 22)
            return ctx.Ebp;
        if (reg == 23)
            return ctx.Esi;
        if (reg == 24)
            return ctx.Edi;
    }
    else {
        CONTEXT ctx = *ctx_proc;
        if (reg == 328)
            return ctx.Rax;
        if (reg == 329)
            return ctx.Rbx;
        if (reg == 330)
            return ctx.Rcx;
        if (reg == 331)
            return ctx.Rdx;
        if (reg == 332)
            return ctx.Rsi;
        if (reg == 333)
            return ctx.Rdi;
        if (reg == 334)
            return ctx.Rbp;
        if (reg == 335)
            return ctx.Rsp;
        if (reg == 336)
            return ctx.R8;
        if (reg == 337)
            return ctx.R9;
        if (reg == 338)
            return ctx.R10;
        if (reg == 339)
            return ctx.R11;
        if (reg == 340)
            return ctx.R12;
        if (reg == 341)
            return ctx.R13;
        if (reg == 342)
            return ctx.R14;
        if (reg == 343)
            return ctx.R15;
    }
    return 0;
}

BOOL CALLBACK CallBack4LocalVar(PSYMBOL_INFO pInfo, ULONG Size, PVOID Context) {
    auto* ctx = static_cast<SymCtxLocal*>(Context);
    auto* symv = ctx->symbolVector;
    HANDLE process = ctx->process;
    LocalSym sd;
    sd.Size = pInfo->Size;
    sd.Value = pInfo->Value;
    sd.Tag = pInfo->Tag;
    sd.Reg = pInfo->Register;
    sd.Name = _strdup(pInfo->Name);
    auto is_wow64 = IsWow64proc(process);
    if (pInfo->Flags & SYMFLAG_REGREL) 
        sd.Address = pInfo->Address + get_reg_value(pInfo->Register, ctx->ctx_proc, is_wow64);
    else if (pInfo->Flags & SYMFLAG_REGISTER)
        sd.Address = get_reg_value(pInfo->Register, ctx->ctx_proc, is_wow64);
    else
        sd.Address = pInfo->Address;

    IMAGEHLP_LINE64 line = { 0 };
    line.SizeOfStruct = sizeof(IMAGEHLP_LINE64);
    DWORD disp = 0;

    if (SymGetLineFromAddr64(process, pInfo->Address, &disp, &line)) {
        sd.line_num = line.LineNumber;
        sd.filename = _strdup(line.FileName ? line.FileName : "Unknown");
    }
    else {
        sd.line_num = 0;
        sd.filename = _strdup("Unknown");
    }
    symv->push_back(sd);
    return TRUE;
}