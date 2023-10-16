// this packages contains a binary crate (this file)
// and a library crate (in lib.rs)
// As recommended in the Rust Book's best practices
// here we use the lib crate
// and we access only the public elements
use learn_wgpu::event_loop::run;

fn main() {
    // tokio could have been also used
    // use pollster to await the futures in run
    pollster::block_on(run());
}
