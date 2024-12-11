# LisaDbg - Debugger

Welcome to **LisaDbg**, a lightweight and powerful debugger written in Rust. Designed for speed and flexibility, LisaDbg offers tailored interfaces for each stage of debugging: initialization, preparation, and execution.

---

## Table of Contents

1. [About the Project](#about-the-project)  
2. [Features](#features)  
3. [Installation](#installation)  
4. [Usage Guide](#usage-guide)  
   - [CLI Interface](#cli-interface)  
   - [Pre-Debugging Interface](#pre-debugging-interface)  
   - [During-Debugging Interface](#during-debugging-interface)  

---

## About the Project

**LisaDbg** is a modern debugger built for a seamless and efficient debugging experience. With intuitive commands and advanced features, LisaDbg is perfect for analyzing and resolving complex programming issues.

---

## Features

- **Multiple Interfaces**:  
  - CLI interface for global options.  
  - Pre-Debugging interface for environment setup.  
  - During-Debugging interface for real-time control.  

- **Flexible Debugging**:  
  - Breakpoints (RVA and VA).  
  - Watchpoints to monitor memory zones.  
  - Symbol loading to work with named functions.  

- **Memory and Execution Flow Inspection**:  
  - Real-time memory state tracking.  
  - Skip function calls effortlessly.  
  - Debug individual threads with precision.  

---

## Installation

### Prerequisites

Ensure you have [Rust](https://rustup.rs/) installed on your system. LisaDbg is only for windows

### Installation via Repository Cloning

1. Clone the Git repository:  
   ```bash
   git clone https://github.com/yourusername/LisaDbg.git
   cd LisaDbg
   ```

2. Build the project:  
   ```bash
   cargo build --release
   ```

3. Run

### Installation via Releases

1. Visit the [Releases Page](https:/r3yl4h/LisaDbg/releases).  
2. Download the latest release.  
3. Extract the downloaded archive and execute `lisa-dbg`.

---

## Usage Guide

LisaDbg provides three main interfaces: **CLI**, **Pre-Debugging**, and **During-Debugging**. Each step is tailored to specific needs.

---

### CLI Interface

Used to initialize LisaDbg, configure options, and launch the target program.  

| **Command**               | **Description**                                                                  |
|---------------------------|----------------------------------------------------------------------------------|
| `-h, --help`              | Displays general help for the debugger.                                          |
| `-V, --version`           | Shows the current version of LisaDbg.                                            |
| `-a, --arg <arg>`         | Sets arguments to pass to the target program.                                    |
| `--attach <pid/name>`     | Attaches LisaDbg to an existing process using its PID or name.                   |
| `--b-ret <address>`       | Places a breakpoint at the return address of a function.                         |
| `--b-ret-va <address>`    | Places a breakpoint at a specific return address (VA).                           |
| `--b-va <address>`        | Sets a breakpoint at a specific address (VA).                                    |
| `-b, --breakpoint <addr>` | Places a breakpoint at a specified address (RVA).                                |
| `--exec <command>`        | Executes a command before starting the debugger.                                 |
| `-w, --watchpoint <zone>` | Sets a watchpoint to monitor a memory zone or register.                          |

---

### Pre-Debugging Interface

Prepare the program before execution: set breakpoints, load symbols, and configure the environment.  

| **Command**               | **Description**                                                                  |
|---------------------------|----------------------------------------------------------------------------------|
| `breakpoint, b`           | Sets a breakpoint (RVA or symbol).                                               |
| `file`                    | Changes the target file for debugging.                                           |
| `run`                     | Starts or resumes execution of the target program.                               |
| `reset`                   | Resets the debugger.                                                             |
| `remove`                  | Removes a breakpoint or other configured element.                                |
| `quit, q, exit`           | Exits LisaDbg.                                                                   |
| `s, sym, symbol`          | Loads symbols to identify functions.                                             |
| `b-ret`                   | Places a breakpoint at a function’s return address.                              |
| `skip`                    | Skips calls to a specified function.                                             |
| `proc-addr`               | Retrieves the address of a function in a DLL.                                    |
| `hook, ho`                | Sets up a function hook to redirect execution.                                   |
| `info`                    | Displays information about breakpoints and loaded symbols.                       |
| `watchpoint, w`           | Sets a watchpoint for a memory address or register.                              |
| `help`                    | Displays help for available commands.                                            |

---

### During-Debugging Interface

Once the program is running, use these commands to control and analyze its behavior.  

| **Command**               | **Description**                                                                  |
|---------------------------|----------------------------------------------------------------------------------|
| `address-func, addr-func` | Displays information about the current function.                                 |
| `backtrace, frame`        | Prints the call stack (backtrace).                                               |
| `base-addr, ba`           | Displays the base address of the target process.                                 |
| `b, breakpoint`           | Sets a breakpoint (RVA or symbol).                                               |
| `b-ret`                   | Sets a breakpoint at a function’s return address.                                |
| `b-va, break-va`          | Sets a breakpoint at a specific address (VA).                                    |
| `continue, c, run`        | Resumes execution of the program.                                                |
| `cva`                     | Calculates a VA from an RVA.                                                     |
| `deref`                   | Dereferences a memory address or register.                                       |
| `dbg-thread, dbg-th`      | Debugs a specific thread by its ID.                                              |
| `disasm`                  | Disassembles instructions from a specific address (VA).                          |
| `find`                    | Searches for a value within a memory range.                                      |
| `help`                    | Displays general help for this interface.                                        |
| `mem-info`                | Provides memory state information for a specific address.                        |
| `proc-addr`               | Retrieves the address of a function in a DLL.                                    |
| `quit, q, break`          | Terminates the debugging session.                                                |
| `reset`                   | Resets the debugger state.                                                       |
| `ret`                     | Sets the instruction pointer (RIP) to the return address of the current function.|
| `set`                     | Modifies registers, values, or memory protection.                                |

---
