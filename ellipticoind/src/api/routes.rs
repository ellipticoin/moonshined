use super::{graphql::handle_graphql, json_rpc::handle_json_rpc};
use crate::api::{blocks, API};
use tide::sse;

impl API {
    pub fn routes(&mut self) {
        self.app
            .at("/static")
            .serve_dir("ellipticoind/static")
            .unwrap();
        self.app.at("/").get(sse::endpoint(blocks::broadcaster));
        self.app.at("/").post(handle_json_rpc);
        self.app.at("/graphql").post(handle_graphql);
    }
}
