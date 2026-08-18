

use tokio::{io::AsyncReadExt, net::{TcpListener, tcp::{OwnedReadHalf, OwnedWriteHalf}}};

use crate::{controller, error::ServerError};

struct server_struct{
    read:OwnedReadHalf,
    write:OwnedWriteHalf,
    controller:controller::controller_struct
}


pub async fn Init(){
    let socket=CreateSocket().await;
    let socket =match socket {
        Ok(socket)=>socket,
        Err(e)=>{
            println!("ServerError:{:?}",e);    
            return ;
        }
    };

    let (mut stream ,client_addr)=match socket.accept().await{
        Ok((stream ,client_addr))=>(stream ,client_addr),
        Err(e)=>{
            let e=ServerError::IO_Error(e);
            println!("ServerError:{:?}",e);    
            return ;
        }
    };

    let (read_stream , write_strema)=stream.into_split();

    let mut server=server_struct{
        read:read_stream,
        write:write_strema,
        controller:controller::controller_struct::new()
    };

    loop{
        let mut buf_len=[0u8;8];
        match server.read.read_exact(&mut buf_len).await {
            Ok(_) => {}

            Err(e) => {
                let e = ServerError::IO_Error(e);
                println!("ServerError: {:?}", e);
                return;
            }
        }

        let len = u64::from_be_bytes(buf_len) as usize;


        let mut payload=vec![0u8;len];

        match server.read.read_exact(&mut payload).await{
            Ok(_) => {}

            Err(e) => {
                let e = ServerError::IO_Error(e);
                println!("ServerError: {:?}", e);
                return;
            }
        };

        let (operation_name_buf,payload)=simplify(payload);


        let buf=HandleOperation(operation_name_buf, payload);


        server.controller.ToExecute(buf);

    }


}   
    
async fn CreateSocket()->Result<TcpListener,ServerError>{
    let addr=std::env::var("docker_addr")?;
    let socket =TcpListener::bind(addr).await?;
    Ok(socket)
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


fn HandleOperation(operation_name_buf:Vec<u8>,payload:Vec<u8>)->Vec<u8>{
    match operation_name_buf.as_slice() {

        b"create_bridge"=>{
            let mut buf=Vec::new();
            let controller_commamnd=b"create_bridge".to_vec();
            let len=(controller_commamnd.len() as u64).to_be_bytes();
            buf.extend_from_slice(&len.to_vec());
            buf.extend_from_slice(&controller_commamnd);

            let mut final_buf=Vec::new();
            final_buf.extend_from_slice(&buf);


            buf.clear();


            let (bridge_name_buf,payload)=simplify(payload);
            let len=(bridge_name_buf.len() as u64).to_be_bytes().to_vec();

            buf.extend_from_slice(&len);
            buf.extend_from_slice(&bridge_name_buf);


            let (bridge_ip,mut payload)=simplify(payload);
            let len=(bridge_ip.len() as u64).to_be_bytes().to_vec();

            buf.extend_from_slice(&len);
            buf.extend_from_slice(&bridge_ip);

            let prefix=payload.pop().unwrap();

            buf.extend_from_slice(&[prefix]);

            let len=(buf.len() as u64).to_be_bytes();
            final_buf.extend_from_slice(&len.to_vec());
            final_buf.extend_from_slice(&buf);

            final_buf
        }

        b"create container"=>{
            let mut final_buf=Vec::new();
            
            
            let (veth_buf,payload)=CreateVethPair(payload.clone());
            let len=(veth_buf.len() as u64).to_be_bytes().to_vec();
            final_buf.extend_from_slice(&len);
            final_buf.extend_from_slice(&veth_buf);
            
            
            let (process_buf,payload)=StartProcess(payload.clone());
            let len=(process_buf.len() as u64).to_be_bytes().to_vec();
            final_buf.extend_from_slice(&len);
            final_buf.extend_from_slice(&process_buf);
            
        





            final_buf
        }

        _=>{
            Vec::new()
        }
    }
}

fn CreateVethPair(payload:Vec<u8>)->(Vec<u8>,Vec<u8>){
    let mut buf=Vec::new();
    let controller_commamnd=b"create_veth".to_vec();
    let len=(controller_commamnd.len() as u64).to_be_bytes();
    buf.extend_from_slice(&len.to_vec());
    buf.extend_from_slice(&controller_commamnd);

    let mut final_buf=Vec::new();
    final_buf.extend_from_slice(&buf);


    buf.clear();


    let (container_name_buf,payload)=simplify(payload);
    let container_name_buf_len=container_name_buf.len() as u64;
    
    let mut veth_front=b"_veth_front".to_vec();
    let len=(veth_front.len() as u64 +container_name_buf_len).to_be_bytes();
    buf.extend_from_slice(&len.to_vec());
    veth_front.extend_from_slice(&container_name_buf);
    buf.extend_from_slice(&veth_front);
    

    let mut veth_back=b"_veth_front".to_vec();
    let len=(veth_back.len() as u64+container_name_buf_len).to_be_bytes();
    buf.extend_from_slice(&len.to_vec());
    veth_back.extend_from_slice(&container_name_buf);
    buf.extend_from_slice(&veth_back);

    
    let len=(buf.len() as u64).to_be_bytes();
    final_buf.extend_from_slice(&len.to_vec());
    final_buf.extend_from_slice(&buf);
    

    (final_buf,payload)

}

fn StartProcess(payload:Vec<u8>)->(Vec<u8>,Vec<u8>){
    let mut buf=Vec::new();
    let controller_commamnd=b"start_process".to_vec();
    let len=(controller_commamnd.len() as u64).to_be_bytes();
    buf.extend_from_slice(&len.to_vec());
    buf.extend_from_slice(&controller_commamnd);

    let mut final_buf=Vec::new();
    final_buf.extend_from_slice(&buf);

    buf.clear();

    let (path_buf,payload)=simplify(payload);
    let len=(path_buf.len() as u64).to_be_bytes().to_vec();
    buf.extend_from_slice(&len);
    buf.extend_from_slice(&path_buf);


    let (arguments_buf,payload)=simplify(payload);
    let len=(arguments_buf.len() as u64).to_be_bytes().to_vec();
    buf.extend_from_slice(&len);
    buf.extend_from_slice(&arguments_buf);

    
    

    final_buf.extend_from_slice(&buf);

    (final_buf,payload)
}
