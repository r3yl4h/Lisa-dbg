---

# **LisaDbg**

LisaDbg is a debugger designed for analyzing and debugging applications. It offers advanced features like breakpoints, watchpoints, attaching to running processes, memory management, and more to assist developers in understanding the internal behavior of a program.

## **Version**
LisaDbg 2.4.0

---

## **Table of Contents**

- [Overview](#overview)
- [Installation](#installation)
- [Usage](#usage)
  - [CLI Commands](#cli-commands)
  - [Debugging Commands](#debugging-commands)
- [Detailed Commands](#detailed-commands)
- [Contributing](#contributing)

---

## **Overview**

LisaDbg is a real-time debugging tool designed to analyze and debug applications. It provides features such as breakpoint management, watchpoints, process attachment, and more. The debugger supports symbolic commands to work with specific functions and addresses.

---

## **Installation**

You can install LisaDbg in one of two ways:

### **Option 1: Clone and Compile the Project**

1. Clone the repository:
   ```bash
   git clone https://github.com/r3yl4h/Lisa-dbg.git
   ```
2. Navigate to the project directory:
   ```bash
   cd Lisa-dbg
   ```
3. Build the project using Cargo:
   ```bash
   cargo build --release
   ```
   This will generate the `LisaDbg` executable in the `target/release/` directory.

### **Option 2: Download from GitHub Releases**

If you prefer not to compile the project, you can download the precompiled release from the GitHub releases page:

1. Go to the [GitHub Releases](https://github.com/r3yl4h/Lisa-dbg/releases) page.
2. Download the appropriate version for your platform (e.g., `.tar.gz` for Linux, `.zip` for Windows).
3. Extract the archive and run the `LisaDbg` executable.

---

## **Usage**

### CLI Commands

When running `LisaDbg` from the command line, you can use the following options and arguments:

#### **Flags:**
- `-h, --help` : Prints help information.
- `-V, --version` : Prints the current version of LisaDbg.

#### **Options:**
- `-a, --arg <arg>` : Set arguments for the script to debug.
- `--attach <attach>` : Attach the debugger to an existing process by PID or name.
- `--b-ret <b-ret>` : Set a breakpoint at the return address of the function.
- `--b-ret-va <b-ret-va>` : Set a breakpoint at the return address of the function which contains the virtual address (VA).
- `--b-va <b-va>` : Set a breakpoint at a specific address (VA) that you know in advance.
- `-b, --breakpoint <breakpoint-addr>` : Set a breakpoint at a specific address (RVA).
- `--exec <exec-cmd>` : Execute a command before running the debugger.
- `-w, --watchpoint <watchpts>` : Set a watchpoint at a specific memory region.

#### **Arguments:**
- `<file>` : The file to debug.

---

### Debugging Commands

During a debugging session, you can use the following commands to interact with the program you are debugging:

#### **General Commands:**
- `breakpoint`, `b` : Set a breakpoint at the specified address (RVA) or symbol.
- `file` : Change the current file context.
- `run` : Start or resume execution of the debugged program.
- `reset` : Reset the debugger settings or context.
- `remove` : Remove a specified element. For more information, type "help remove".
- `quit`, `q`, `exit` : Exit the debugger.
- `s`, `sym`, `symbol` : Load symbols, which allows commands like `b-ret` to be used with function names directly.
- `b-ret` : Set a breakpoint at the return address of the specified function.

#### **Advanced Commands:**
- `hook`, `ho` : Set up a function hook to redirect execution flow.
- `create-func`, `crt-func` : Create a custom function with a return value allocated at runtime.
- `printf` : `printf` displays a formatted string by replacing specifiers (%d, %s, etc.) with the values of the provided variables or arguments.
- `info` : View certain information like the symbols and breakpoints that have been placed.

#### **Commands During Debugging:**
- `address-func`, `addr-func` : Display current function information.
- `backtrace`, `frame` : Print the call stack frames for debugging purposes.
- `mem-info` : Display all memory information at this address (base address, state, etc.).
- `deref` : Dereference the value at a specific memory address or register in the target process.
- `disasm` : Disassemble opcodes from a specified address (VA) or symbol name.
- `continue`, `c`, `run` : Continue the execution of the process.
- `set` : To set something, it can be a register, a value at an address, or a memory protection. For more information, type "help set".

---

## **Detailed Commands**

### **Element-Specific Commands**

Some commands are specific to an element of debugging, such as registers, threads, or symbols. You can get detailed information by typing:

```bash
help all <element-name>
```

The available elements include:
- `b, break, breakpoint`
- `run`
- `reset, remove`
- `ret`
- `thread, th`
- `register, reg`

For detailed usage, just type `help <command>` without its arguments.

---

## **Contributing**

If you wish to contribute to LisaDbg, feel free to open issues or submit pull requests. Ensure that you follow coding best practices and include tests for any new features.

---
