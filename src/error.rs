use std::{env::VarError, fmt, io};



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

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::VarError(e) => {
                write!(f, "Environment variable error: {}", e)
            }

            ServerError::IO_Error(e) => {
                write!(f, "IO error: {}", e)
            }

            ServerError::RT_net_Error(e) => {
                write!(f, "RTNetlink error: {}", e)
            }

            ServerError::InterfaceNotFound(name) => {
                write!(f, "Interface not found: {}", name)
            }
        }
    }
}

impl std::error::Error for ServerError {}