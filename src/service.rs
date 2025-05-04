use tonic::{Request, Response, Status};

use executor::code_executor_server::CodeExecutor;
use executor::{ExecuteRequest, ExecuteResponse};
// Removed invalid macro usage as it is not required for the current implementation.
// pub mod executor {
//     tonic::include_proto!("executor");
// }
pub mod executor {
    tonic::include_proto!("executor");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("executor_descriptor");
}
#[derive(Debug, Default)]
pub struct ExecutorService;

#[tonic::async_trait]
impl CodeExecutor for ExecutorService {
    async fn execute(
        &self,
        _request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let reply = ExecuteResponse {
            message: "Request received".into(),
        };
        Ok(Response::new(reply))
    }
}