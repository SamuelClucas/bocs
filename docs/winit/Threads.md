When a program runs, it becomes a 'process'.

A process has a 'context' that holds its metadata for use in its code, or for the os when switching focus to another process (whose context is used to configure that process...).

If a process contains context, and it also contains code, where's the action? When does **stuff** happen?

That is the **thread**: the active part of a process, undergoing live computation. The things itself moves through your application, like a diligent little agent-of-operations. 
- moves through functions
- follows 'control $\mathcal{flowww...}$'
- manages the 'stack'

In this *context*, you begin to understand that an application's context itself is a descriptor that informs the OS of the how, what, why, when, and where, from which it births (or revives) the thread. 
	Context switching uses both the suspended program's context to put a thread to rest, and the approached program's context to inaugurate or revive' the correct thread on the CPU.

When you run a Rust program, the OS breathes life into a **main thread** that runs off down into your `main()` function. When this little thread calls a function (i.e., begins processing a function using a CPU core), it's referred to as the 'calling thread' of a function.

##### Now, back to serious business:
The thread is literally-speaking a data-structure managed by the OS, stored in its kernel—i.e., its brain. It represents a point of execution in a process' code using a 'program counter'. It also has:
- CPU register state (which values are in which registers)
- its very own stack (coming to this next...)
- access to shared memory of a process

### The stack: built frame-by-frame

Each **stack frame** is like a page in that notebook:
- **Function args**
- **Local variables**
- **Return address** (i.e., _where to resume_ in the caller after `ret`, related to program counter)
- **Saved frame pointer / stack pointer** (so the CPU knows how to walk back a frame when this is done!)

This structure is maintained by **calling conventions** (how you call a function-like object, for example) — CPU- and language-defined rules for:
- How arguments are passed (e.g. registers or stack)
- Where return values go
- How to clean up the stack afterward

> `Function call` → push frame  
> `Return` → pop frame  
> → Resumes at return address with restored pointer values

---
### One Stack Per Thread
Each **thread** gets its **own private stack**, typically pre-allocated with a fixed size (often 1MB by default).
- Distinct threads don't share! (unlike heap or global memory)
- There’s **no race condition** on local variables between threads (at least they don't argue)
- You can have recursion and nested function calls scoped to a thread

It’s like a **notebook**:  
Only _that_ thread can scribble in its own.

---
###  Clarification: Return Address ≠ "Memory to Write To"

Just a small but key precision:
The **return address** stored in the stack frame is the **instruction pointer** (e.g. `RIP` on x86-64) — the location in code to jump back to after the function is done.

The **value** returned might:
- Go into a register (`eax` on x86, for instance)
- Or be written into memory passed in by reference
- Or copied by move or clone semantics (e.g., in Rust)

So:
- Return address → _control flow_ (resume execution)
- Return value → _data flow_ (passed into a register, pointer, etc.)

Again, 
- **control flow:** relates to the 'execution point' of a process, where that little thread *daemon* has managed to get to 
- data flow: flow of data between frames, for example

**Bonus definition!**
*To invoke: to cause a procedure to be carried out.*