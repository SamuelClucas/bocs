There are times, particularly when programming an app with a graphical backend, that you will want a particular 'task/ to continue in the background—that is, not in the main thread.

For example, when requesting a `Device` using wgpu's `Adapter`, you don't want to block that main thread (imagine the poor little frustrated daemon desperate to continue its work)—and indeed, it hasn't been designed that way, anyway. 

It uses a Rust concept called a `Future`, which is just as it sounds. 
	It's a type used for any procedure (think any function-like object) that might take some unknown duration to 'complete', so you want to defer its evaluation, lest you impede your main thread (doing so would be akin to the stopping your app's heart for a moment. Don't do that).

Typically it's the OS kernel that regulates execution of however many active threads your system currently has (all competing for cores on that CPU). Think of this kernel much like a caring mother, multi-tasking, trying to give all her little *daemon's* their due computation time. She wouldn't want any of them to feel left out!

In Rust, a `Future` isn't a kernel-level thread—it’s a state machine  (implementing the `Future` trait) as a struct or enum in your code. It doesn’t do anything on its own; it needs a runtime to poll it to completion.
	*Bonus definitions:*
		To poll: to check the status of a system, especially during part of a repeated cycle. For example, a runtime that calls `.poll()` on each loop cycle.

`Futures` can be `await`ed in the body of `async` functions only (this is important, and sometimes a pain as we will see). When an `async fn` is called, it returnsreturns a `Future`, which is a suspended computation. Nothing inside the async function runs yet—it must be explicitly `.await`ed (or polled by the runtime) to begin. If you never `.await`, it never runs. 
	This is what is really meant by async programming—handling the non-sequential execution of code. That doesn't necessarily mean concurrency (more on concurrency when we get around to shaders...). Async allows for _non-blocking_ execution—it defers work, but doesn't imply multiple things happen _at once_.
When `.await` is called, if the async code can be executed immediately (like an addition, for example), then the calling thread will evaluate it there and then. If this isn't the case– 
	— (i.e., if nothing changes, the code being executed will block the calling thread from making any further progress, called a 'hang') —
- then the procedure is managed by our process/runtime's 'scheduler' or 'executor'. This way, the calling thread may continue with its sequential business. This is called 'yielding': the async code yields to allow this 'driver'/'reactor' to continue without any blocking on the main thread (which is managed by the OS). The driver also notifies the scheduler when a yielded process is ready to be scheduled (i.e., it's no longer blocking)
	*Terminology note*: terms like process, runtime, application—they may feel vague or interchangeable. That's because they pretty much are. They're fundamentally quite abstract, but in their essence, they're the living entity that is your program. They comprise the 'executor' and the 'driver'.

To summarise runtimes:
Executor/scheduler: the state machine responsible for scheduling the async code for evaluation
Driver: the state machine providing non-blocking alternatives to the async code to prevent handing. Also notifies the scheduler when async tasks are no longer blocking.
	I've tried to represent the system in a hierarchy below...

## OS
- **Threads** 
	- stack
	- program counter
	- cpu register state
##### Application/Runtime/Process
- Code
- Environmental infos/context
- event loop(OS interface, event dispatch—each beat of execution)
- **scheduler**...

The scheduler is what allows for asynchronous programming. You can see above that threads are managed by the OS. The scheduler manages the timetable of your application's execution. Where the event loop specifies which code needs to be executed, the scheduler determines when and *where* it will be executed. By where, I mean on which OS-managed thread. Imagine a train station worker, blowing their whistle and pointing here and there, ensuring trains leave on-time.

So far, we've discussed futures and async. But there's no real concurrency yet, that is, multiple distinct procedures co-occurring. So far, there's just waiting and altered main thread behaviour—but still all sequential.

A task is within the app's runtime. Each `tokio::spawn` call creates a lightweight, managed task that the tokio runtime schedules across its thread pool (if using the multi-threaded runtime). The spawn function returns a JoinHandle (you don't need to use it), which can be awaited alongside other spawned tasks which will all run concurrently.  
Tokio _can_ achieve true concurrency—but only if it’s configured to do so and your machine has multiple cores. Tokio supports **multi-threaded** runtimes. The default (in recent versions) is the _multi-threaded scheduler_, which will spawn a thread pool that can genuinely run multiple tasks in parallel, depending on system constraints.

However, Tokio tasks (spawned via `tokio::spawn`) are **not OS threads**—they are _green threads_, or _cooperative tasks_, managed in userspace by the Tokio scheduler. So:

- Tokio tasks can **run in parallel** across OS threads if the runtime is configured to use more than one worker thread (`#[tokio::main(flavor = "multi_thread", worker_threads = 4)]`)
    
- But even if you're using the **current-thread flavour**, Tokio tasks are still scheduled intelligently to avoid blocking.
    

So yes, Tokio can achieve real concurrency on the CPU, even without GPU involvement. But only when the runtime is configured that way and the task is **CPU-bound and send + sync safe.** Otherwise, it's "concurrent" in a cooperative sense, not a preemptive one.


CHALLENGES TODAY: 
Traits in Rust do not allow you to modify the signatures of the associated functions when implementing the trait for a struct. It's very much so akin to a header file in C++, the definition has to match. Winit's `ApplicationHandler` doesn't have an async function i.e., a context in which I can call `.await`. I'm currently using the resume() function to setup the graphical backend within a conditional block. On the first call to resume, the block is executed, guarded by a bool flag in App that prevents repeated setup on every call to resume. When setting up the graphical backend via wgpu, I run into issues with all the requests, namely: 
- `ActiveEventLoop::RequestAdapter()`
- `ActiveEventLoop::RequestDevice()`
These both return Futures, and remember: **you can only call `.await()` in an async context**, and the ApplicationHandler trait just doesn't have an async function.
I did not want to implement the future trait for App. I do not want the app struct itself to be a state machine that can poll futures. The app is for reactivity, user interfacing, and gpu computation. I decided it was cleaner to isolate the async runtime from App using `tokio::spawn`, which creates a `Task` in which I await the requests in a non-blocking manner. I capture everything i need from app using `move` to shift ownership inside the async code block. I then await the requests, and inject the result back into app via UserEvent proxy. I smuggled the proxy into app in main on `EventLoop::run_app()`. Another option would be to define my own trait with an async function using `#[async_trait::async_trait]`. This boxes the future for you, where a box is a smart pointer for heap allocation (required for things like Futures that can't exist on the stack). I would still have to send back via UserEvent, but it might be cleaner than leaving the setup code inside `App::resume()`.

# In Summary:
> **How do you await a `Future` in a system that doesn’t support `async fn`s?**

In my case, Winit’s `ApplicationHandler` trait expects synchronous methods like `resumed(&mut self, ...)`, so I couldn't just slap `async` on it. And WGPU’s `request_adapter` and `request_device` methods return `Futures`—_they must be awaited_. So I needed:

- A way to **spawn async work outside of that trait-bound method**
- A way to **get results back into your app** on the main thread

My solution:

- `tokio::spawn` for the async work
- `EventLoopProxy` to inject the result back into the Winit-managed event loop (and into my App)

That’s _the canonical pattern_. Libraries like [`egui_winit`](https://github.com/emilk/egui) or [`bevy`](https://bevyengine.org/) have to work around the same boundaries. 