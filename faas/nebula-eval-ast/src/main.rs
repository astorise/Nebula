#[cfg(unix)]
use std::path::Path;

#[cfg(unix)]
use nebula_eval_ast::proto::ast_evaluator_server::{AstEvaluator, AstEvaluatorServer};
#[cfg(unix)]
use nebula_eval_ast::proto::{EvaluationRequest, EvaluationResponse};
#[cfg(unix)]
use nebula_eval_ast::{evaluate_proto_request, DEFAULT_SOCKET_PATH};
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;
#[cfg(unix)]
use tonic::{transport::Server, Request, Response, Status};

#[cfg(unix)]
#[derive(Default)]
struct AstEvaluatorService;

#[cfg(unix)]
#[tonic::async_trait]
impl AstEvaluator for AstEvaluatorService {
    async fn evaluate_triplets(
        &self,
        request: Request<EvaluationRequest>,
    ) -> Result<Response<EvaluationResponse>, Status> {
        Ok(Response::new(evaluate_proto_request(request.into_inner())))
    }
}

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path =
        std::env::var("NEBULA_AST_SOCKET").unwrap_or_else(|_| DEFAULT_SOCKET_PATH.to_string());

    if Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    let incoming = UnixListenerStream::new(listener);

    Server::builder()
        .add_service(AstEvaluatorServer::new(AstEvaluatorService))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}

#[cfg(not(unix))]
fn main() {
    eprintln!("nebula-eval-ast microVM server requires a Unix Domain Socket runtime");
}
