use opcua_server::prelude::*;
use opcua_server::constants;
use std::sync::{Arc, Mutex};

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let path: &str ="/";
    let ns: u16;

    let user_token_ids = [
        ANONYMOUS_USER_TOKEN_ID,
    ]
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<String>>();

    let mut server: Server = ServerBuilder::new()
        .application_name("producer")
        .application_uri("urn:producer")
        .product_uri("urn:producer")
        .discovery_server_url(Some(constants::DEFAULT_DISCOVERY_SERVER_URL.to_string()))
        .endpoints(vec![("none", ServerEndpoint::new_none(path, &user_token_ids))])
        .discovery_urls(vec![path.into()])
        .server().unwrap();
    let address_space_arc = server.address_space();

    {
        let mut address_space = address_space_arc.write().unwrap();

        ns = address_space.register_namespace("urn:demo-server").unwrap();

        let static_folder_id = address_space
            .add_folder("Static", "Static", &NodeId::objects_folder_id())
            .unwrap();
        let scalar_folder_id = address_space
            .add_folder("Scalar", "Scalar", &static_folder_id).unwrap();

        VariableBuilder::new(&NodeId::new(ns, "Test"), "Test", "Test")
            .data_type(DataTypeId::UInt32)
            .value(0u8)
            .organized_by(&scalar_folder_id)
            .writable()
            .insert(&mut address_space);
    }

    let data: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    server.add_polling_action(100, move || {
        let mut data = data.lock().unwrap();
        *data += 1;
        let mut address_space = address_space_arc.write().unwrap();
        let now = DateTime::now();
        address_space.set_variable_value(&NodeId::new(ns, "Test"), *data, &now, &now);
        println!("{}", *data);
    });

    server.run();
}
