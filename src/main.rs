mod state;
mod event_loop;
use event_loop::run;

fn main() {
    // tokio could have been also used
    // use pollster to await the futures in run
    pollster::block_on(run());
}
