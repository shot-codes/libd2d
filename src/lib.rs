use std::time::Duration;
use futures::task::Poll;
use futures::task::Context;
use std::thread;
use futures::task::Waker;
use async_std::stream::Stream;
use core::pin::Pin;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ndarray::Array2;
use libp2p::PeerId;
use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub struct MothershipState {
    pub position: Coordinate,
    pub mission_status: MissionStatus,
    pub mission_area: Option<Array2<u32>>,
    pub tasks: Arc<Mutex<VecDeque<Coordinate>>>,
    pub delegate_tasks: DelegateTasks,
}

#[derive(Debug)]
pub struct MinionState  {
    pub heartbeat: bool,
    pub ready: bool,
    pub position: Coordinate,
    pub poi: bool,
    pub mission_area: Option<Array2<u32>>,
    pub waker: Option<Waker>,
}

#[derive(Debug)]
pub struct MinionHeartbeat {
    pub position: Coordinate,
    pub poi: bool,
}

#[derive(Debug)]
pub struct MinionStream {
    shared_state: Arc<Mutex<MinionState>>,
}

#[derive(Debug)]
pub enum MissionStatus {
    Pending,
    InProgress,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn inc_x(&mut self) {
        self.x = self.x + 1;
    }
}

// Struct used by mothership to keep track of minions
#[derive(Debug, Serialize, Deserialize)]
pub struct Minion {
    pub peer_id: PeerId,
    pub position: Coordinate,
}

#[derive(Debug)]
pub struct DelegateTasks {
    pub minions: HashMap<PeerId, Coordinate>,
    pub total: u32, // This is set once the mission is received, based on the number of subscribed minions.
    pub complete: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelegateTaskMessage {
    pub peer_id: PeerId,
    pub area: Array2<u32>,
}

impl Stream for MinionStream {

    type Item = MinionHeartbeat;

    fn poll_next(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {

        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.ready {

            if shared_state.heartbeat {
                shared_state.heartbeat = false;
                return Poll::Ready(Some(MinionHeartbeat {
                    position: shared_state.position.clone(),
                    poi: shared_state.poi.clone(),
                }));
            } else {
                shared_state.waker = Some(cx.waker().clone());
                return Poll::Pending;
            }
        } else {
            shared_state.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
    }
}

impl MinionStream {

    pub fn new(shared_state: Arc<Mutex<MinionState>>) -> Self {

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let mut shared_state = thread_shared_state.lock().unwrap();

                // Advance to next position
                shared_state.position.inc_x();
                if shared_state.position.x % 2 == 0 {
                    shared_state.poi = true;
                } else {
                    shared_state.poi = false;
                };
                // Tell comms to poll again.
                shared_state.heartbeat = true;
                if let Some(waker) = shared_state.waker.take() {
                    waker.wake()
                }
            }
        });

        MinionStream { shared_state }
    }
}

pub fn mothership_bot (tasks: Arc<Mutex<VecDeque<Coordinate>>>) {
    loop {
        let mut tasks = tasks.lock().unwrap();
        if let Some(task) = tasks.pop_front() {
            drop(tasks);
            println!("Running pick up on {:?}", task);
            // Do pickup with robot
        } else {
            drop(tasks);
            println!("No more tasks");
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

// pub fn minion_bot (tasks: Arc<Mutex<VecDeque<Array2<u32>>>>, points_of_interest:Arc<Mutex<VecDeque<Coordinate>>>) {
//     loop {
//         let mut tasks = tasks.lock().unwrap();
//         if let Some(task) = tasks.pop_front() {
//             drop(tasks);
//             println!("Running search on {:?}", task);
//             // Do search with robot
//             {
//                 let mut pois = points_of_interest.lock().unwrap();
//                 pois.push_front(Coordinate {x:0, y:0});
//             }
//         } else {
//             drop(tasks);
//             println!("No more tasks");
//         }

//         std::thread::sleep(std::time::Duration::from_secs(5));
//     }
// }
