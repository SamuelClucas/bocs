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
- init.rs – Adapter, device, and surface selection. Surface format/present mode configuration.
- resources.rs – Creation of world-sized GPU resources (uniform buffers, voxel buffers, samplers).
- frame_targets.rs – Creation and rebuild of window-sized targets (swapchain, storage texture) and their bind groups.
- shaders.rs – WGSL module loading utilities.