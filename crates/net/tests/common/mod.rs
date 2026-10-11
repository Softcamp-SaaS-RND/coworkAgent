#![allow(dead_code)]
use an_core::{db, paths::Paths};
use an_net::identity::Identity;
use an_net::node::{Me, Node, PAIRING_WINDOW};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Running {
    pub node: Arc<Node>,
    pub addr: SocketAddr,
    pub paths: Paths,
}

/// 임시 데이터 루트 · 실제 신원 키 · 실제 DB 로 127.0.0.1 임의 포트에 수신하는 노드.
pub async fn start(tag: &str) -> Running {
    let root = std::env::temp_dir().join(format!("an-net-node-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = Paths::under(&root);
    paths.ensure().unwrap();
    let id = Identity::load_or_create(&paths.identity_file()).unwrap();
    let conn = db::open(&paths.db_file()).unwrap();
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let me = Me {
        name: format!("user-{tag}"),
        host: format!("PC-{tag}"),
        port: addr.port(),
    };
    let node = Node::new(id, conn, me);
    tokio::spawn(node.clone().serve(listener));
    Running { node, addr, paths }
}

/// 신원 키만 있는 대역 — 노드 없이 요청을 보낼 때(임의 키 32+개).
pub fn bare_identity() -> Identity {
    let pkcs8 =
        ring::signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();
    Identity::from_pkcs8(pkcs8.as_ref().to_vec()).unwrap()
}

impl Running {
    /// 노드와 같은 DB 파일의 다른 연결 — 노드를 거치지 않고 상태를 만들 때.
    pub fn db(&self) -> an_core::db::Connection {
        db::open(&self.paths.db_file()).unwrap()
    }

    /// "페어링 받기" — 처음 보는 키의 hello 1건. 초대 코드를 돌려준다.
    pub fn open(&self) -> Option<String> {
        Some(self.node.open_window(PAIRING_WINDOW))
    }

    pub fn only_peer(&self) -> an_core::peers::Peer {
        let mut ps = self.node.peers().unwrap();
        assert_eq!(ps.len(), 1, "{ps:?}");
        ps.remove(0)
    }
}

/// 손으로 만드는 hello — 세대는 고정값.
pub fn hello(name: &str, host: &str, port: u16, code: Option<String>) -> an_net::proto::Request {
    an_net::proto::Request::Hello {
        name: name.into(),
        host: host.into(),
        port,
        round: "r-test".into(),
        code,
    }
}
