use crate::engine::db::HashMapDb;
use crate::protocol;

pub fn execute_command(
    cmd: protocol::commands::Command,
    db: &mut HashMapDb,
    now: std::time::Instant,
) -> protocol::commands::CommandResponse {
    match cmd {
        protocol::commands::Command::Get { key } => protocol::commands::CommandResponse::String {
            value: db
                .get(key.as_str(), std::time::Instant::now())
                .map_or("not found", |v| v)
                .to_owned(),
        },
        protocol::commands::Command::Set { key, value, ttl } => {
            db.set(key.as_str(), value, ttl.map(|ttl| now + ttl));
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
        protocol::commands::Command::Del { key } => {
            db.del(key.as_str());
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
        protocol::commands::Command::Exists { key } => {
            let exists = db.exists(key.as_str(), now);
            protocol::commands::CommandResponse::Integer {
                value: i64::from(exists),
            }
        }
        protocol::commands::Command::Docs => {
            protocol::commands::CommandResponse::Array { value: Vec::new() }
        }
        protocol::commands::Command::Config => protocol::commands::CommandResponse::String {
            value: "OK".to_owned(),
        },
        protocol::commands::Command::Ping => protocol::commands::CommandResponse::String {
            value: "PONG".to_owned(),
        },
        protocol::commands::Command::Incr { key } => {
            match db.get(&key, std::time::Instant::now()) {
                Some(k) => {
                    let k = k.parse::<u64>().unwrap();
                    db.set(&key, (k + 1).to_string(), None);
                }
                None => db.set(&key, 1.to_string(), None),
            }

            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
        protocol::commands::Command::FlushDb => {
            db.flush();
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;

    #[test]
    fn exec_get() {
        let mut db = HashMapDb::new(config::Engine::default());
        db.set("key", "value".to_string(), None);

        let cmd = protocol::commands::Command::Get {
            key: "key".to_string(),
        };
        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::String {
                value: "value".to_owned()
            }
        );
    }

    #[test]
    fn exec_exists() {
        let mut db = HashMapDb::new(config::Engine::default());
        db.set("key", "value".to_string(), None);

        let cmd = protocol::commands::Command::Exists {
            key: "key".to_string(),
        };
        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::Integer { value: 1 }
        );
    }

    #[test]
    fn exec_incr() {
        let mut db = HashMapDb::new(config::Engine::default());

        // incr with no key
        let cmd = protocol::commands::Command::Incr {
            key: "key".to_string(),
        };

        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned()
            }
        );

        let v = db
            .get("key", std::time::Instant::now())
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert_eq!(v, 1);

        // incr with key
        let cmd = protocol::commands::Command::Incr {
            key: "key".to_string(),
        };
        let res = execute_command(cmd, &mut db, std::time::Instant::now());
        assert_eq!(
            res,
            protocol::commands::CommandResponse::String {
                value: "OK".to_owned()
            }
        );

        let v = db
            .get("key", std::time::Instant::now())
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert_eq!(v, 2);
    }
}
