use derive_more::Display;
use randao::error::{Error, Result};
use redis::Client;

#[derive(Debug, Display)]
#[display(fmt = "{}, {}, {:?}", proto, endpoint, client)]
pub struct Db {
    endpoint: String,
    proto: Proto,
    client: Client,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Proto {
    Url,
    Unix,
}

impl std::fmt::Display for Proto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let proto = match self {
            Self::Unix => "unix socket",
            Self::Url => "redis",
        };
        write!(f, "{}", proto)
    }
}

impl Db {
    /// connect to a new redis server
    pub fn new(
        proto: Option<Proto>,
        auth: Option<(&str, &str)>,
        path: &str,
        port: Option<u32>,
        db: Option<u8>,
    ) -> Result<Self> {
        let proto = proto.unwrap_or(Proto::Url);
        let endpoint = match proto {
            Proto::Url => {
                let mut endpoint = "redis://".to_string();
                if let Some((user, passwd)) = auth {
                    endpoint.push_str(format!("{}:{}@", user, passwd).as_str());
                }
                endpoint.push_str(path);
                if let Some(port) = port {
                    endpoint.push_str(format!(":{}", port).as_str());
                }
                if let Some(db) = db {
                    endpoint.push_str(format!("/{}", db).as_str());
                }
                endpoint
            }
            Proto::Unix => return Err(Error::NotSupport("Unix socket is not supported currently".to_string())),
        };

        Ok(Self {
            proto,
            client: Client::open(endpoint.as_str())?,
            endpoint,
        })
    }

    /// insert a data
    pub fn insert(&self, key: u64, data: &[u8]) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        Ok(redis::cmd("SET").arg(key).arg(data).query(&mut conn)?)
    }

    /// get a data
    pub fn get(&self, key: u64) -> Result<String> {
        let mut conn = self.client.get_connection()?;
        let res: String = redis::cmd("GET").arg(key).query(&mut conn)?;
        Ok(res)
    }
}
