# GPU

This directory modularises all WGPU-related configuration (as in [state.rs](../state.rs)) to improve extensibility, clarify intent at call sites, and reduce boilerplate bulk.  

It achieves this by combining:  
- Small, descriptive enums – used to represent configuration choices (e.g., buffer access mode, storage texture access, uniform usage) instead of raw booleans or “option soup.”  
    - This enables polymorphism through pattern matching — the builder can branch internally depending on the variant.  
- Struct builders – used to accumulate configuration via method chaining, producing final WGPU objects only when .build() is called.  
    - This keeps creation code declarative, and prevents copy-paste of verbose descriptor structures.

## Overview of Modules  
- enums.rs – Core configuration enums (StorageTex, BufferAccess, UniformUsage, …).
    - These form the vocabulary for describing resource and pipeline properties.
- builders.rs – Builder types (BindGroupLayoutBuilder, PipelineBuilder, etc.) that accept enums, accumulate state, and produce WGPU objects.
- traits.rs - Pipeline trait declaration for shared Compute/Render behaviour.
- compute.rs - defines the Compute struct for management of Compute pipeline.
- render.rs - defines the Render struct for management of Render pipeline.
- resources.rs - defines the Resource struct responsible for managing bind group resources.
- gfx_context.rs - defines the GraphicsContext struct responsible for managing wgpu handles to like `Device`.