use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent},
    futures::StreamExt,
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    NetworkBehaviour, PeerId, Transport,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{io::AsyncBufReadExt, sync::mpsc};

use crate::modules::PeerCounter::PeerCounter;

//Generating Peer Key
static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());

//Generating Peer ID to locate and reference a peer
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));

//Some messages to send and recieve
#[derive(Debug, Serialize, Deserialize)]
enum ListMode {
    ALL,
    One(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct ListRequest {
    mode: ListMode,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    mode: ListMode,
    data: String,
    receiver: String,
}

enum EventType {
    Response(ListResponse),
    Input(String),
    PeerConnected(PeerId), //Event for when a new peer connects
    PeerDisconnected(PeerId), //Event for when a peer disconnects
}

#[derive(NetworkBehaviour)]
struct GameBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
    #[behaviour(ignore)]
    response_sender: mpsc::UnboundedSender<ListResponse>,
    
}


//Network Behaviour configuration for Floodsub Events implemented on GameBehaviour Struct
impl NetworkBehaviourEventProcess<FloodsubEvent> for GameBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(msg) => {
                if let Ok(resp) = serde_json::from_slice::<ListResponse>(&msg.data) {
                    if resp.receiver == PEER_ID.to_string() {
                        info!("Response from {}:", msg.source);
                        
                    }
                } else if let Ok(req) = serde_json::from_slice::<ListRequest>(&msg.data) {
                    match req.mode {
                        ListMode::ALL => {
                            info!("Received ALL req: {:?} from {:?}", req, msg.source);
                          
                        }
                        ListMode::One(ref peer_id) => {
                            if peer_id == &PEER_ID.to_string() {
                                info!("Received req: {:?} from {:?}", req, msg.source);
                                
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
}


//Network Behavior configuration for mDNS messages implemented on Game Behaviour struct
impl NetworkBehaviourEventProcess<MdnsEvent> for GameBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}


//Main function for the p2p client 
#[tokio::main]
pub async fn p2pclient() {
    pretty_env_logger::init();

    info!("Peer Id: {}", PEER_ID.clone());
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();

    //Generate the Authentication Keypairs for peer connection
    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&KEYS)
        .expect("can create auth keys");

    //Generate Transport channel for peer connection
    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated()) // XX Handshake pattern, IX exists as well and IK - only XX currently provides interop with other libp2p impls
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    //Initialise Game Behaviour for peer connection 
    let mut behaviour = GameBehaviour {
        floodsub: Floodsub::new(PEER_ID.clone()),
        mdns: Mdns::new(Default::default())
            .await
            .expect("can create mdns"),
        response_sender,
    };


    //Initialise Swarm for peer connection
    let mut swarm = SwarmBuilder::new(transp, behaviour, PEER_ID.clone())
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();
    
    let mut num_peers = 0;


    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    loop {

        let evt: Option<EventType> = {
            tokio::select! {
                line = stdin.next_line() => Some(EventType::Input(line.expect("can get line").expect("can read line from stdin"))), //Handle req
                response = response_rcv.recv() => Some(EventType::Response(response.expect("response exists"))),//send res
                event = swarm.select_next_some() => {
                    info!("Unhandled Swarm Event: {:?}", event);
                    None
                },
            }
        };

        let mut peer_counter = PeerCounter::new();



        if let Some(event) = evt {
            match event {
                EventType::Response(resp) => {
                    
                    println!("Response Generation here..")

                }
                EventType::Input(line) => match line.as_str() {
                    "ls p" =>  handle_list_peers(&mut swarm).await,
                    _ => error!("unknown command"),
                },

                EventType::PeerConnected (peer_id) => {
                    // Handle new connection established
                    println!("New connection established with peer: {:?}", peer_id);
                    peer_counter.increment();
                    println!("Number of connected peers: {}", num_peers);
                }
                EventType::PeerDisconnected (PeerId ) => {
                    // Handle connection closed
                    println!("Connection closed with peer: {:?}", PeerId);
                    peer_counter.decrement();
                    println!("Number of connected peers: {}", num_peers);
                }

                
            }
        }
    }
}

//function for listing the peers connected to the current node
async fn handle_list_peers(swarm: &mut Swarm<GameBehaviour>) {

    info!("Discovered Peers:");
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }

    unique_peers.iter().for_each(|p| info!("{}", p));


      
}