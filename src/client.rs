// TODO(client-api)
// - Define operations: Put(key, value, request_id), Get(key)
// - Decide input/output types (simple vs request/response structs)
// - Define error types (timeout, leader_unavailable, network_drop, other)
// - Define Put status (committed vs pending)
// - Define RPC boundary (leader-only trait)
// - Define retry policy fields (max_attempts, sleep_between)


// our default client
pub struct Client<Rpc, Retry> {
    pub rpc: Rpc,
    pub retry: Retry,
}

// Explicit request/response types (no logic yet)
// just want to specify all our request here 
pub struct PutRequest {
    pub key: String,
    pub value: String,
    pub request_id: String,
}

pub struct PutResponse {
    pub status: PutStatus,
}

pub struct GetRequest {
    pub key: String,
}

pub struct GetResponse {
    pub value: Option<String>,
}


pub enum PutStatus{
    Committed,
    Pending
}



// possible errors that may occur either from the 
// rpc or the client.
pub enum RpcError{
    Timeout,
    NetworkDrop,
    LeaderUnavailable,
    Other(String)
}

pub enum ClientError{
    Timeout,
    LeaderUnavailable,
    Rpc(String)
}

pub trait LeaderRpc{
    fn put(&self, req: PutRequest)->Result<PutResponse, RpcError>;
    fn get(&self, req: GetRequest)->Result<GetResponse, RpcError>;
}


pub struct RetryPolicy{
    pub max_attempts: usize,
    pub sleep_between_ms: u64,
}

impl<Rpc,Retry>Client<Rpc,Retry>{
    pub fn put(&self, req:PutRequest)->Result<PutResponse, ClientError>{
        unimplemented()
    }
    
    pub fn get(&self, req:GetRequest)->Result<GetResponse, ClientError>{
        unimplemented()
    }

    
}




