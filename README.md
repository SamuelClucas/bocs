# Digi-Lab  
<img width="400" height="400" alt="logo" src="https://github.com/user-attachments/assets/d6b95cba-d50a-4863-978d-981848007c51" /><br />
## A 3D scientific simulation engine... built with Rust! 
  
### Vision:
With the advent of deep machine learning techniques giving rise to tools like [AlphaFold](https://alphafold.ebi.ac.uk/), challenges that once seemed impossible - like de novo protein design - now enter into feasibility. It's only natural to imagine what else might be possible.  
  
I am greatly excited by the idea of synthetic cell design. Imagine a cell whose genome contains 'modules' that confer particular functionalities or cell behaviours. There are also great implications for the design of new, organic materials, with unique properties that arise from biological mechanisms. One such idea I had was a salt cell: one that uses an Electron Transport Chain/Resting potential hybrid mechanism (as in a chloroplast's thylakoid, and a neuronal cell) to use saltwater to store chemical energy.  
  
There's a whole lot of science that needs to happen to realise cool things like this — I also understand that wetlab experimentation is expensive, fiddly, and time-consuming...  
  
I ask myself this: how can I contribute?  
  
### My Skills:  
- programming and project documentation
- understanding of mathematical fundamentals
- awareness of physics and chemistry
- BSc 1:1 in biological sciences from Durham University

### My Interests:  
I am fascinated by the 'what?' and 'how?' of things. More importantly, I won't stop chasing the answers - my attempt at control amidst the unrelenting chaos of the universe.  
  
I believe I am uniquely positioned to design an _in-silico_ cell simulation platform - wetlab isn't my strongest asset...  
In particular, **I am interested in modelling membrane physics at the mesoscopic scale**, prioritising both **performance** and **realism**. To do so, I am switching from my beloved C++ to Rust. The language enforces solid program design, is memory-safe, and also incredibly performant.  

> [!CAUTION]
> This is going to be a real challenge.  

I do not want to use a high-level graphical library or game-engine, like [`bevy`](https://bevy.org/learn/book/getting-started/), for example. I want to fully own my code, which empowers me as a researcher in a few ways:  
1. I intimately understand the tool because I built it
2. Because of the above, I understand its limitations
3. In Digi-Lab, I am god. I can continue to develop its features, hopefully converging on something special...
  
For these reasons, I am writing the simulation engine in [`wgpu`](https://docs.rs/wgpu/latest/wgpu/), Rust's flagship graphics library. Its documentation is robust, as is its code, from which I hope to learn and become a better systems architect. I plan to take advantage of parallel computation (GPU-acceleration) wherever practical to expand the engine's capabilities as much as possible.  
