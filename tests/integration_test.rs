use discovery::{DataCenterInfo, EurekaClient, Instance, LeaseInfo, PortData, Result, StatusType};
use std::env;

fn get_client() -> EurekaClient {
    let host = env::var("EUREKA_HOST").unwrap_or("localhost".into());
    dbg!(&host);
    EurekaClient::new(format!("http://{}:8761/eureka/apps", host))
}

fn register_dummy() -> Result<()> {
    let c = get_client();
    let inst = Instance {
        host_name: "127.0.0.1".into(),
        app: "myapp".into(),
        ip_addr: "127.0.0.1".into(),
        vip_address: "myapp.example.com".into(),
        secure_vip_address: Some("myapp.example.com".into()),
        status: StatusType::Up,
        port: Some(PortData::new(8787, true)),
        secure_port: PortData::new(8787, true),
        home_page_url: "http://127.0.0.1:8787/".into(),
        status_page_url: "http://127.0.0.1:8787/status".into(),
        health_check_url: "http://127.0.0.1:8787/helth".into(),
        data_center_info: DataCenterInfo::default(),
        lease_info: Some(LeaseInfo {
            eviction_duration_in_secs: None,
        }),
        metadata: None,
    };

    c.register("myapp", &inst)
}

#[test]
fn integration_client_register() {
    let res = register_dummy();
    assert!(res.is_ok(), "{:?}", res.err());
    // calling twice still should work
    let res = register_dummy();
    assert!(res.is_ok(), "{:?}", res.err());
}

#[test]
fn integration_get_all() {
    let c = get_client();
    register_dummy().unwrap();
    // app might not be available immediately
    let mut result = false;
    for i in 0..10 {
        let res = c.get_all();
        assert!(res.is_ok(), "{:?}", res.err());
        let res = res.unwrap();
        dbg!(&i);
        if res.applications.get("MYAPP").is_some() {
            result = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    assert!(result, "The returned apps state doesn't have MYAPP");
}

#[test]
fn integration_register_and_get() {
    register_dummy().unwrap();

    let c = get_client();
    let mut result = false;
    for i in 0..10 {
        let res = c.get("MYAPP");
        if res.is_ok() {
            result = true;
            let res = res.unwrap();
            assert!(res.len() == 1);
            break;
        }
        dbg!(&i);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    assert!(result);
}
