use std::{collections::{HashMap, HashSet}, ffi::CString, mem, net::Ipv4Addr};


use ipnetwork::Ipv4Network;
use rand::RngExt;

use crate::{IP_pool::{self, IPpool}, controller::{self, controller_struct}};




pub struct manager_struct{
    controller:controller::controller_struct,
   
    IPpool:HashMap<String,IP_pool::IPpool>,
    active_pool:IP_pool::IPpool,

}



impl manager_struct {
    pub async fn new()->Self{
        let controller=controller_struct::new();
        
        let (ippool,buf)=IPpool::new("default".to_string(),Ipv4Addr::new(10, 2, 0, 1),"dc-br0".to_string(),24);
        controller.ToExecute(buf).await;
        let mut  v=HashMap::new();
        v.insert("default".to_string(),ippool.clone());
        Self{
            controller,
            IPpool:v,
            active_pool:ippool
        }
        
    }

   
    fn GetRandomIp(&self, network: Ipv4Network) -> Ipv4Addr {
        let start = u32::from(network.network());
        let end = u32::from(network.broadcast());

        let mut rng = rand::rng();

        loop {
            let ip = Ipv4Addr::from(rng.random_range(start..=end));

            if !self.active_pool.ips_allocated.contains(&ip) {
                return ip;
            }
        }
    }

    async fn HandleOperation(&mut self,operation:Vec<u8>,payload:Vec<u8>){

        match operation.as_slice() {
            b"start_process"=>{
                


                let (process_buf,payload)=simplify(payload);
                let path;
                let args;
                {
                    let (path_buf, payload) = simplify(process_buf);

                    path = CString::new(str::from_utf8(&path_buf).unwrap()).unwrap();

                    let (args_buf, _payload) = simplify(payload);

                    let args_string = String::from_utf8(args_buf).unwrap();

                    let arg_cstrings: Vec<CString> = args_string
                        .split_whitespace()
                        .map(|arg| CString::new(arg).unwrap())
                        .collect();

                    let mut argv: Vec<*const libc::c_char> = Vec::new();

                    argv.push(path.as_ptr());

                    for arg in &arg_cstrings {
                        argv.push(arg.as_ptr());
                    }

                    argv.push(std::ptr::null());

                    args = argv;

                }

                let (veth_buf,payload)=simplify(payload);
                {   
                    let mut final_buf=Vec::new();

                    let controller_commamnd=b"create_veth".to_vec();
                    let len=(controller_commamnd.len() as u64).to_be_bytes();
                    final_buf.extend_from_slice(&len.to_vec());
                    final_buf.extend_from_slice(&controller_commamnd);


                    let len=(veth_buf.len() as u64).to_be_bytes().to_vec();
                    final_buf.extend_from_slice(&len);
                    final_buf.extend_from_slice(&veth_buf);

                    self.controller.ToExecute(final_buf).await;
                }
                let (veth_front_buf,veth_payload)=simplify(veth_buf);
                let (veth_back_buf,_)=simplify(veth_payload);
                {
                    let mut final_buf=Vec::new();

                    let controller_commamnd=b"assign_veth_ip".to_vec();
                    let len=(controller_commamnd.len() as u64).to_be_bytes();
                    final_buf.extend_from_slice(&len.to_vec());
                    final_buf.extend_from_slice(&controller_commamnd);


                    let len=(veth_back_buf.len() as u64).to_be_bytes().to_vec();
                    final_buf.extend_from_slice(&len);
                    final_buf.extend_from_slice(&veth_back_buf);


                    let bridge_network=self.active_pool.network;
                    let ip=self.GetRandomIp(bridge_network);

                    self.active_pool.ips_allocated.insert(ip.clone());

                    let ip_buf=ip.to_string().as_bytes().to_vec();
                    let len=(ip_buf.len() as u64).to_be_bytes().to_vec();
                    final_buf.extend_from_slice(&len);
                    final_buf.extend_from_slice(&ip_buf);

                    self.controller.ToExecute(final_buf).await;
                }

                let pid=CreateChildProcess();

                if pid ==-1{
                    println!("Error while creating a child preocess");
                    return;
                }
                // child execution
                if pid==0{
                    //gateway adn vethsend to toconfigure teh rules
                    let buf={
                        let mut final_buf=Vec::new();

                        let controller_commamnd=b"assign_veth_ip".to_vec();
                        let len=(controller_commamnd.len() as u64).to_be_bytes();
                        final_buf.extend_from_slice(&len.to_vec());
                        final_buf.extend_from_slice(&controller_commamnd);


                        let len=(veth_back_buf.len() as u64).to_be_bytes().to_vec();
                        final_buf.extend_from_slice(&len);
                        final_buf.extend_from_slice(&veth_back_buf);

                        let gateway=self.active_pool.gateway_to_bridge.clone();
                        let gateway_buf=gateway.to_string().as_bytes().to_vec();
                        let len=(gateway_buf.len() as u64).to_be_bytes().to_vec();
                        final_buf.extend_from_slice(&len);
                        final_buf.extend_from_slice(&gateway_buf);

                        final_buf
                    
                    };
                    self.controller.ToExecute(buf).await;


                    // unsafe {
                    //     libc::execve(
                    //         path.as_ptr(),
                    //         args.as_ptr(),
                    //         std::ptr::null(),
                    //     );
                    // }
                }
                //dc execution
                else{
                    let pid: u32 = pid.try_into().expect("PID does not fit into u32");
                    let buf={
                        let mut final_buf=Vec::new();

                        let controller_commamnd=b"move_veth_to_netns_by_pid".to_vec();
                        let len=(controller_commamnd.len() as u64).to_be_bytes();
                        final_buf.extend_from_slice(&len.to_vec());
                        final_buf.extend_from_slice(&controller_commamnd);


                        let len=(veth_back_buf.len() as u64).to_be_bytes().to_vec();
                        final_buf.extend_from_slice(&len);
                        final_buf.extend_from_slice(&veth_back_buf);

                        let pid_buf = pid.to_be_bytes();
                        let len=(pid_buf.len() as u64).to_be_bytes().to_vec();
                        final_buf.extend_from_slice(&len); 
                        final_buf.extend_from_slice(&pid_buf);   

                        final_buf

                    };
                    self.controller.ToExecute(buf).await;
                }

            }

            

            _=>{

            }
        }
    }
}


fn simplify(mut payload: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let len_bytes: [u8; 8] = payload[..8]
        .try_into()
        .unwrap();

    let len = u64::from_be_bytes(len_bytes) as usize;

    let acquired = payload[8..8 + len].to_vec();

    let remaining = payload[8 + len..].to_vec();

    (acquired, remaining)
}

fn CreateChildProcess()->i64{
    let pid=unsafe {
        let mut args: libc::clone_args = mem::zeroed();

        args.flags =
            libc::CLONE_NEWNS      as u64 |
            libc::CLONE_NEWUTS     as u64 |
            libc::CLONE_NEWIPC     as u64 |
            libc::CLONE_NEWPID     as u64 |
            libc::CLONE_NEWNET     as u64 |
            libc::CLONE_NEWUSER    as u64 |
            libc::CLONE_NEWCGROUP as u64 |
            libc::CLONE_NEWTIME    as u64;

        args.exit_signal = libc::SIGCHLD as u64;

        let pid = libc::syscall(
            libc::SYS_clone3,
            &mut args,
            mem::size_of::<libc::clone_args>(),
        );

        pid
    };
    pid
}


