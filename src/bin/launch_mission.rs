use futures::{prelude::*, select};
use libp2p::gossipsub::{ GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode };
use libp2p::{gossipsub, identity, swarm::SwarmEvent, Multiaddr, PeerId};
use std::error::Error;
use std::time::Duration;
#[macro_use(array)]
extern crate ndarray;
use serde_json;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic_new_mission = Topic::new("new_mission");

    let mut swarm = {

        // Set a custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .duplicate_cache_time(Duration::from_secs(1))
            .build()
            .expect("Valid config");

        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                .expect("Correct configuration");

        gossipsub.subscribe(&topic_new_mission).unwrap();

        // build the swarm
        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    // Dial mothership
    let address: Multiaddr = "/ip4/127.0.0.1/tcp/60740".parse().unwrap();
    match swarm.dial(address.clone()) {
        Ok(_) => println!("Dialed {:?}", address),
        Err(e) => println!("Dial {:?} failed: {:?}", address, e),
    };

    loop {
        select! {
            event = swarm.select_next_some() => match event {

                // Once we know we have subscribers, send the mission
                SwarmEvent::Behaviour(GossipsubEvent::Subscribed {
                    topic: t, 
                    ..
                })  if t == topic_new_mission.hash() => {

                        let mission_area: ndarray::Array2<u32> = array![[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0]];
                    // let mission_area: ndarray::Array2<u32> = array![[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    //                                                 [0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    //                                                 [0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    //                                                 [0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0],
                    //                                                 [0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0],
                    //                                                 [0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
                    //                                                 [0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0],
                    //                                                 [0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0],
                    //                                                 [0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0],
                    //                                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0],
                    //                                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0],
                    //                                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12],
                    //                                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];

                    let serialized = serde_json::to_string(&mission_area).unwrap();

                    if let Err(e) = swarm
                        .behaviour_mut()
                        .publish(topic_new_mission.clone(), serialized.as_bytes())
                    {
                        println!("Publish error: {:?}", e);
                    };
                },

                _ => {}
            }
        }
    }

}