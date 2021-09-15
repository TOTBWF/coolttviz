mod linalg;
mod cube;
mod render;
mod system;
mod messages;
mod server;
mod camera;

use messages::Message;

fn main() {
    render::display_hypercube(4);
    // server::server(8080, |msg| {
    //     match msg {
    //         Message::DisplayGoal(goal) => render::display_goal(goal)
    //     }
    // })
}
