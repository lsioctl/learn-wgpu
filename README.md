# learn-wgpu

Personnal testing around WGPU in Rust.

99% comes from:

https://sotrh.github.io/learn-wgpu

# differences from the official website

For what I understand, WebGPU will use Vulkan, DirectX, Metal or
WebGL depending on the target architecture.

I think WebGPU is a nice candidate to code once for all the platforms,
and I would like to see if there are easily noticeable performances issues
without taking care of the browser's WebGPU support.

So I removed the few lines targeting WASM (and this is powerfull too, only
a few lines !)

Another difference is that I also try to learn Rust and how to make a proper Rust project,
with code splitted in multiple files and so on.