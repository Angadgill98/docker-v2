use std::{env::VarError, io};



#[derive(Debug)]
pub enum ServerError{
    VarError(VarError),
    IO_Error(io::Error),
    RT_net_Error(rtnetlink::Error),
    InterfaceNotFound(String)
}

impl From<VarError> for ServerError {
    fn from(value: VarError) -> Self {
        ServerError::VarError(value)
    }
}

impl From<io::Error> for ServerError {
    fn from(value: io::Error) -> Self {
        ServerError::IO_Error(value)
    }
}

impl From<rtnetlink::Error> for ServerError {
    fn from(value: rtnetlink::Error) -> Self {
        ServerError::RT_net_Error(value)
    }
}

