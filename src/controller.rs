
use std::{fs::OpenOptions, io::Write, net::{IpAddr, Ipv4Addr}};

use futures_util::TryStreamExt;
use rtnetlink::{Handle, LinkBridge, LinkUnspec, LinkVeth, packet_route::RouteNetlinkMessage, proto::Connection};

use crate::error::ServerError;

use rtnetlink::{ RouteMessageBuilder};
pub struct controller_struct{
    rtnetlink:Handle,
}

impl controller_struct {
    pub fn new() -> Self {
        let (connection, handle, _) =
            rtnetlink::new_connection().unwrap();

        tokio::spawn(connection);

        Self {
            rtnetlink: handle,
        }
    }
    
    pub async fn ToExecute(&self, buf: Vec<u8>) {
        // println!("{:?}", buf);

        let mut remaining = buf;

        while !remaining.is_empty() {
            if remaining.is_empty() {
                break;
            }
            let (command_buf, payload) = simplify(remaining);

            let (command_payload, payload) = simplify(payload);

            remaining = payload;

            self.HandleCommands(command_buf, command_payload).await;
        }
    }

    async fn GetIndex(&self,interface_name:String)->Result<u32,ServerError>{
        let mut links = self
        .rtnetlink
        .link()
        .get()
        .match_name(interface_name.clone())
        .execute();

        let link = links
            .try_next()
            .await
            .map_err(ServerError::RT_net_Error)?
            .ok_or(ServerError::InterfaceNotFound(interface_name))?;

        let index = link.header.index;

        Ok(index)
    }

    async fn HandleCommands(&self,command_buf:Vec<u8>,payload:Vec<u8>){
        self.WrtitToLog(command_buf.clone(), payload.clone());
        match command_buf.as_slice() {

            b"create_bridge"=>{
                let(bridge_name_buf,payload)=simplify(payload);

                let bridge_name=String::from_utf8(bridge_name_buf).unwrap();

                let linkmsg=LinkBridge::new(&bridge_name).build();

                match self.rtnetlink.link()
                .add(linkmsg)
                .execute()
                .await
                {
                    Ok(())=>{}
                    Err(e)=>{
                        let e=ServerError::RT_net_Error(e);
                        println!("failed to create bridge :{}",e);
                        return ;
                    }
                }
                let index=match self.GetIndex(bridge_name).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("faile tofi{}",e);
                        return ;
                    }
                };


                
            }

            b"assign_ip_bridge"=>{

                let(bridge_name_buf,payload)=simplify(payload);

                let bridge_name=String::from_utf8(bridge_name_buf).unwrap();

                let index=match self.GetIndex(bridge_name.clone()).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("falied to find index of bridge {}",e);
                        return ;
                    }
                };


                let(bridge_ip_buf,mut payload)=simplify(payload);

                let ip:IpAddr=str::from_utf8(&bridge_ip_buf).unwrap().parse().unwrap();
                


                let prefix = payload[0];
                println!("{} {} {} {}",bridge_name,ip,prefix ,index);
                match self.rtnetlink
                    .address()
                    .add(index, ip.into(), prefix)
                    .execute()
                    .await
                {
                    Ok(_) => {
                        // IP assigned successfully
                    }

                    Err(e) => {
                        let e=ServerError::RT_net_Error(e);
                        println!("failed to assing ip  {}",e);
                        return ;
                    }
                }

            }
            //for both veth adn bridge
            b"up_interface"=>{
                let(bridge_name_buf,payload)=simplify(payload);

                let bridge_name=String::from_utf8(bridge_name_buf).unwrap();

                let index=match self.GetIndex(bridge_name).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("failed to being he interafec up {}",e);
                        return ;
                    }
                };



                match self.rtnetlink
                    .link()
                    .set(
                        LinkUnspec::new_with_index(index)
                            .up()
                            .build()
                    )
                    .execute()
                    .await

                {
                    Ok(a)=>{}
                    Err(e)=>{
                        let e=ServerError::RT_net_Error(e);
                        println!("{}",e);
                        return ;
                    }

                }
            }

            b"create_veth"=>{
                let(veth_front_name_buf,payload)=simplify(payload);
                let veth_front=str::from_utf8(&veth_front_name_buf).unwrap();
                
                
                let(veth_back_name_buf,payload)=simplify(payload);
                let veth_back=str::from_utf8(&veth_back_name_buf).unwrap();
            
                
                match self.rtnetlink
                    .link()
                    .add(
                        LinkVeth::new(veth_front, veth_back).build()
                    )
                    .execute()
                    .await
                {
                    Ok(())=>{}
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                }
            }
            
            b"assign_veth_ip"=>{
                let(veth_back_name_buf,payload)=simplify(payload);
                let veth_back=str::from_utf8(&veth_back_name_buf).unwrap();
                
                let index=match self.GetIndex(veth_back.to_string()).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                };

                let(veth_ip_buf,mut payload)=simplify(payload);

                let ip:Ipv4Addr=str::from_utf8(&veth_ip_buf).unwrap().parse().unwrap();
                


                let prefix=payload.pop().unwrap();

                match self.rtnetlink
                    .address()
                    .add(
                        index,
                        IpAddr::V4(ip),
                        prefix,
                    )
                    .execute()
                    .await 
                {
                    Ok(())=>{}
                    Err(e)=>{
                        
                        println!("{}",e);
                        return ;
                    }
                }
            }

            b"assign_mac_to_veth"=>{

                let(veth_back_name_buf,payload)=simplify(payload);
                let veth_back=str::from_utf8(&veth_back_name_buf).unwrap();
                
                let index=match self.GetIndex(veth_back.to_string()).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                };
                let (mac,payload)=simplify(payload);

                self.rtnetlink.link()
                .set(LinkUnspec::new_with_index(index).down().build())
                .execute()
                .await?;
                

                self.rtnetlink.link()
                .set(LinkUnspec::new_with_index(index).address(mac).build())
                .execute()
                .await?;

                self.rtnetlink
                .link()
                .set(LinkUnspec::new_with_index(index).up().build())
                .execute()
                .await?;
            }

            b"move_veth_to_netns_by_pid"=>{
                let(veth_name_buf,payload)=simplify(payload);
                let veth_name=str::from_utf8(&veth_name_buf).unwrap();
                
                let index=match self.GetIndex(veth_name.to_string()).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                };


                let(container_pid_buf,payload)=simplify(payload);
                let container_pid = u32::from_be_bytes(
                    container_pid_buf
                        .try_into()
                        .expect("PID must be 4 bytes")
                );



                

                match self.rtnetlink
                    .link()
                    .set(LinkUnspec::new_with_index(index)
                        .setns_by_pid(container_pid)
                        .build()
                    )
                    .execute()
                    .await
                {
                    Ok(())=>{}
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                }
            }
            //to be excuted in the child itlsef
            b"add_veth_as_default"=>{
                let(veth_name_buf,payload)=simplify(payload);
                let veth_name=str::from_utf8(&veth_name_buf).unwrap();
                
                let index=match self.GetIndex(veth_name.to_string()).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                };
                let(veth_ip_buf,mut payload)=simplify(payload);

                let gateway_ip:Ipv4Addr=str::from_utf8(&veth_ip_buf).unwrap().parse().unwrap();
                
                
               
                let route_msg = RouteMessageBuilder::<Ipv4Addr>::new()
                    .destination_prefix(Ipv4Addr::UNSPECIFIED, 0)
                    .gateway(gateway_ip)
                    .output_interface(index)
                    .build();

                match self.rtnetlink.route()
                    .add(route_msg)
                    .execute()
                    .await
                {
                    Ok(())=>{}
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                }





            }

            b"delete_veth"=>{
                let(veth_name_buf,payload)=simplify(payload);
                let veth_name=str::from_utf8(&veth_name_buf).unwrap();
                
                let index=match self.GetIndex(veth_name.to_string()).await{
                    Ok(index)=>{
                        index
                    }
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                };
                 match self.rtnetlink.link()
                .del(index)
                .execute()
                .await
                {
                    Ok(())=>{
                        
                    }
                    Err(e)=>{
                        println!("{}",e);
                        return ;
                    }
                }
            }
            
            b"start_process"=>{

            }

            b"restart_container_process"=>{

            }

            b"start_container_process"=>{
                
            }

            b"stop_container_process"=>{
                
            }

            b"delete_container_process"=>{

            }



            _=>{

            }
        }
    }

    fn WrtitToLog(&self,command_buf:Vec<u8>,payload:Vec<u8>){
        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("server.log").unwrap();

        let command_len = command_buf.len() as u64;
        let payload_len = payload.len() as u64;

        // command length
        file.write_all(&command_len.to_be_bytes()).unwrap();

        // command
        file.write_all(&command_buf).unwrap();

        // payload length
        file.write_all(&payload_len.to_be_bytes()).unwrap();

        // payload
        file.write_all(&payload).unwrap();

        // newline
        file.write_all(b"\n").unwrap();

      
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

