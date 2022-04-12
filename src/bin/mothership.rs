use std::collections::VecDeque;
use std::error::Error;
use std::thread;
use std::sync::{Arc, Mutex};
use async_std::task;

use libd2d::{Mission, MothershipState, MissionStatus, Area, Point, DelegateTasks, mothership_bot};
use libd2d::comms::create_p2p_network;


#[async_std::main]

async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Set initial state
    let state = MothershipState {
        mission: Mission {
            status: MissionStatus::InProgress,
            area: Area {x1: 0, y1: 0, x2: 10, y2: 10},
        },
        tasks: Arc::new(Mutex::new(VecDeque::from([
            Point {x:1, y:1}, 
            Point {x:2, y:2}, 
            Point {x:3, y:3},
        ]))),
        delegate_tasks: DelegateTasks {
            total: 0,
            complete: 0,
        }

    };

    // create robot thread
    let tasks = Arc::clone(&state.tasks);
    let robot_handle = thread::spawn(move || mothership_bot(Arc::clone(&state.tasks)));

    // create libp2p thread
    let comm_handle = task::spawn(create_p2p_network());


    let mut tasks = tasks.lock().unwrap();
    tasks.push_back(Point {x:4, y:4});
    println!("{:?}", state);
    drop(tasks);

    // Prevent main from exiting while thread is running
    robot_handle.join().unwrap();
    comm_handle.await?;

    Ok(())

}
