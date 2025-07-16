### Firstly, what the f\*ck is a context?
When a program is run, it may be better described as a 'process' that uses the CPU to perform computations as it runs—or any other component it needs to function.

An operating system must manage process access to a CPU, lest the process orchestra turn into the process rave. 

Processes at any time have data associated with them: pointers, values in CPU registers, current state etc. This can be thought of as the broader environment a process needs to thrive (or indeed, to function). 
	Fittingly, operating systems represent this *environment* as a 'context'.

For a multi-tasking operating system managing several processes at once, it must *switch* between these contexts as the user shifts their focus from one *process* (i.e. a running program) to another. 

This is called a **'context switch'**:
	it involves tidily encoding an up-to-date description of the process now going out of focus (a good sort of 'context' hygiene) before loading the context of the process to which the focus is shifting.

In `winit`, a _context_ is programmatically represented by an `EventLoop`, a construct that manages interaction between the system and the application. 
	This can be thought of as a useful 'logical' representation of the *actual* context, abstracted away from OS-dependent idiosyncrasies that would otherwise make platform-agnostic development a pain in the arse.

But before using this logical context to create a window, we need an application handler — a kind of listener or orchestrator for user events. 
	The human-machine interfacer.

In this model, the **user interacts with the application** (e.g. by clicking), which in turn maintains and reacts through its **context** — the structures that represent its interface with the OS and graphical backend.

User → \[ OS → (Application (with its context, and code)$_n$) ] → \[ Drivers →( Components)$_n$]
Where $n = \text{however many}$
\[ = interface boundary
( = child

These are **user events**, the lifeblood of reactive computation. They shape the _feel_ of a program:

- Is it responsive?
    
- Does it change colour?
    
- What happens when I click here?

An event dispatcher is the responding element to an event. It sets in motion the signals required for the system to respond accordingly—a central node and its corresponding listeners. In winit, the `EventLoop` can be thought of as the interface with the OS kernel, channeling events to your `App`'s event handler for dispatch (using `EventHandler`... all to come :p)

User Activity (from a device...)→ \[ event channeling in OS Kernel → (App's EventLoop interface →( EventHandler Dispatcher )→ Listeners (could be in the program, back to the kernel for channeling to other components, it's up to you!)

This brings us onto another foundational concept... [[Threads]]!