mod linalg;
mod cube;
mod render;
mod system;
mod messages;
mod server;

use messages::Message;

fn main() {
    server::server(8080, |msg| {
        match msg {
            Message::DisplayGoal(goal) => render::display_goal(goal)
        }
    })
}
