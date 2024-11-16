---

# **LisaDbg**

LisaDbg is a debugger designed for developers to analyze and debug programs effectively. It provides advanced features like symbolic debugging, memory observation, breakpoints, and real-time process manipulation. LisaDbg is flexible, efficient, and supports debugging of running processes or standalone files.

---

## **Version**
**LisaDbg 2.4.0**

---

## **Table of Contents**

1. [Overview](#overview)
2. [Installation](#installation)
   - [Option 1: Clone the Repository](#option-1-clone-the-repository)
   - [Option 2: Download from Releases](#option-2-download-from-releases)
3. [Usage](#usage)
   - [CLI Options](#cli-options)
   - [Commands Before Debugging](#commands-before-debugging)
   - [Commands During Debugging](#commands-during-debugging)
4. [Contributing](#contributing)

---

## **Overview**

LisaDbg is a robust debugger that helps you inspect, debug, and analyze application behavior. Its features include:
- **Breakpoint management:** Add or remove breakpoints at specific addresses or functions.
- **Memory observation:** Watch memory locations for changes.
- **Symbolic debugging:** Work with functions and symbols for easier debugging.
- **Process attachment:** Attach to and debug running processes.

Whether you’re debugging a file or attaching to a live process, LisaDbg gives you the tools you need to understand and fix issues.

---

## **Installation**

LisaDbg can be installed in two ways:

### **Option 1: Clone the Repository**

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
4. The executable will be available in the `target/release/` directory.

---

### **Option 2: Download from Releases**

1. Go to the [Releases](https://github.com/r3yl4h/Lisa-dbg/releases) page.
2. Download the appropriate binary for your platform.
3. Extract the archive, and you’re ready to use LisaDbg.

---

## **Usage**

LisaDbg operates through three main interfaces:
1. **Command-Line Interface (CLI)** – Used to start the debugger with specific options or arguments.
2. **Commands Before Debugging** – Configure the debugger or program context.
3. **Commands During Debugging** – Control and analyze the target program after execution begins.

---

### **CLI Options**

Use the following options to start LisaDbg:

#### **Usage**
```bash
LisaDbg [OPTIONS] [--] [file]
```

#### **Flags**
- `-h, --help` : Show help information.
- `-V, --version` : Display the version of LisaDbg.

#### **Options**
- `-a, --arg <arg>` : Set arguments for the script to debug.
- `--attach <attach>` : Attach to an existing process by PID or name.
- `--b-ret <b-ret>` : Add a breakpoint at the return address of the function containing the specified RVA.
- `--b-ret-va <b-ret-va>` : Add a breakpoint at the return address of the function containing the specified VA.
- `--b-va <b-va>` : Add a breakpoint at a specific virtual address (VA).
- `-b, --breakpoint <breakpoint-addr>` : Add a breakpoint at a specific relative virtual address (RVA).
- `--exec <exec-cmd>` : Execute a command before running the debugger.
- `-w, --watchpoint <watchpts>` : Set a watchpoint in the format `[--memory=<zone>] [--access=<rights>] <offset>`.

#### **Arguments**
- `<file>` : Specify the file to debug.

---

### **Commands Before Debugging**

When LisaDbg is launched, you can configure and prepare the debugger using these commands:

| Command                    | Description |
|----------------------------|-------------|
| `breakpoint, b`            | Add a breakpoint at the specified RVA or symbol. |
| `file`                     | Change the current file context. |
| `run`                      | Start or resume the program execution. |
| `reset`                    | Reset debugger settings or context. |
| `remove`                   | Remove a specified element (e.g., breakpoint). |
| `quit, q, exit`            | Exit the debugger. |
| `s, sym, symbol`           | Load symbols to enable symbolic debugging. |
| `b-ret`                    | Add a breakpoint at the return address of a function or instruction. |
| `skip`                     | Skip calls to the specified function. |
| `proc-addr`                | Retrieve the address of a function in a DLL. |
| `hook, ho`                 | Set up a function hook to redirect execution flow. |
| `create-func, crt-func`    | Create a custom function with a return value allocated at runtime. |
| `info`                     | Display information (e.g., placed breakpoints). |
| `watchpoint, watch, w`     | Set a memory observation point. |
| `sym-info`                 | Show detailed symbol information. |
| `arg, args, argv`          | Set program arguments for the debugged program. |
| `attach`                   | Attach the debugger to a running process. |
| `break-va, b-va`           | Add a breakpoint at a specified VA. |
| `break-ret-va, b-ret-va`   | Add a breakpoint at the return address of a VA-specified function. |
| `printf`                   | Display formatted strings. |
| `help, h`                  | Display help for commands. |

For detailed usage, type:
```bash
help <command>
```

---

### **Commands During Debugging**

Once execution begins and a breakpoint is hit, you can use the following commands:

| Command                    | Description |
|----------------------------|-------------|
| `address-func, addr-func`  | Show current function information. |
| `backtrace, frame`         | Print the call stack. |
| `base-addr, ba`            | Display the process’s base address. |
| `b, breakpoint`            | Add a breakpoint at the specified RVA or symbol. |
| `b-ret`                    | Add a breakpoint at a function's return address. |
| `b-va, break-va`           | Add a breakpoint at a specific VA. |
| `continue, c, run`         | Resume program execution. |
| `cva`                      | Convert an RVA to a VA. |
| `deref`                    | Dereference a memory address or register. |
| `dbg-thread, dbg-th`       | Debug a specific thread by its ID. |
| `disasm`                   | Disassemble opcodes at a specific VA. |
| `find`                     | Search for a value between two addresses. |
| `mem-info`                 | Show memory information (e.g., base address, state). |
| `proc-addr`                | Retrieve a function's address in a DLL. |
| `quit, q, break`           | Terminate the debugging session. |
| `reset`                    | Reset the debugger state. |
| `ret`                      | Set the instruction pointer (RIP) to the current function's return address. |
| `set`                      | Set values (e.g., register, memory protections). |
| `skip`                     | Skip calls to a specified function. |
| `s`                        | Load symbol files if available. |
| `sym-address`              | Display the address of a symbol by name. |
| `symbol-local, sym-local`  | Display all local symbols for the current function. |
| `thread-info, th-info`     | Show information about the current thread. |
| `value, v, register, reg, r` | Display a register's value. |
| `info`                     | Show information like active breakpoints, active thread, etc. |
| `printf`                   | Print formatted output. |

To learn all commands for an element:
```bash
help all <element-name>
```

---

## **Contributing**

Contributions are welcome! Submit issues or pull requests to help improve LisaDbg. Be sure to include tests and documentation for any new features.

---
