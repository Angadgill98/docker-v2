use std::{collections::HashSet, net::Ipv4Addr};

use ipnetwork::Ipv4Network;




#[derive(Clone)]

pub struct IPpool{
    pub network_name:String,
    pub network:Ipv4Network,
    pub gateway_to_bridge:Ipv4Addr,
    pub bridge_name:String,
    pub ips_allocated:HashSet<Ipv4Addr>
}


struct container{}

impl container {
    pub fn new()->Self{
        Self{ 
            
        }
    }
}


impl IPpool {
    pub fn new(name:String,birdge_ip:Ipv4Addr,bridge_name:String,prefix:u8)->(Self,Vec<u8>){
        
        let (buf,mut a) =CreateNetwork(name, bridge_name, birdge_ip.clone(), prefix);

        a.ips_allocated.insert(birdge_ip);

        (a,buf)
    }
}

fn CreateNetwork(network_name:String,bridge_name: String,gateway: Ipv4Addr,prefix: u8,) -> (Vec<u8>, IPpool) {
    let mut buf=Vec::new();

    let mut payload = Vec::new();

    // create_bridge
    let command = b"create_bridge";

    payload.extend_from_slice(&(command.len() as u64).to_be_bytes());
    payload.extend_from_slice(command);



    let bridge_name_buf = bridge_name.as_bytes();

    buf.extend_from_slice(&(bridge_name_buf.len() as u64).to_be_bytes());
    buf.extend_from_slice(bridge_name_buf);


    payload.extend_from_slice(&(buf.len() as u64).to_be_bytes());
    payload.extend_from_slice(&buf);

    
    buf.clear();
    
    
    let command=b"up_interface";
    payload.extend_from_slice(&(command.len() as u64).to_be_bytes());
    payload.extend_from_slice(command);


    buf.extend_from_slice(&(bridge_name_buf.len() as u64).to_be_bytes());
    buf.extend_from_slice(bridge_name_buf);

    payload.extend_from_slice(&(buf.len() as u64).to_be_bytes());
    payload.extend_from_slice(&buf);

    
    buf.clear();


    // assign_ip_bridge
    let command = b"assign_ip_bridge";

    payload.extend_from_slice(&(command.len() as u64).to_be_bytes());
    payload.extend_from_slice(command);


    // bridge name
    buf.extend_from_slice(&(bridge_name_buf.len() as u64).to_be_bytes());
    buf.extend_from_slice(bridge_name_buf);

    // IP
    let gateway_string = gateway.to_string();
    let gateway_buf = gateway_string.as_bytes();

    buf.extend_from_slice(&(gateway_buf.len() as u64).to_be_bytes());
    buf.extend_from_slice(gateway_buf);

    // prefix
    buf.push(prefix);

    payload.extend_from_slice(&(buf.len() as u64).to_be_bytes());
    payload.extend_from_slice(&buf);
    


    // Network
    let subnet = Ipv4Network::new(gateway, prefix).unwrap();

    let mut pool = IPpool {
        network_name,
        network:subnet,
        gateway_to_bridge:gateway,
        bridge_name,
        ips_allocated: HashSet::new(),
    };

    

    (payload, pool)
}


