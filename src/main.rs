mod linalg;
mod cube;
mod render;
mod system;
mod messages;
mod server;
mod camera;
mod vertex;

use messages::Message;

fn main() {
    // render::display_hypercube(vec!["i".to_string(), "j".to_string()]);
    render::display_hypercube(vec!["i".to_string(), "j".to_string(), "k".to_string()]);
    // server::server(8080, |msg| {
    //     match msg {
    //         Message::DisplayGoal(goal) => render::display_goal(goal)
    //     }
    // })
}
