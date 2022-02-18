use opcua_client::prelude::*;
use std::sync::{Arc, RwLock};

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let mut client: Client = ClientBuilder::new()
        .application_name("consumer")
        .application_uri("urn:Consumer")
        .create_sample_keypair(true)
        .trust_server_certs(false)
        .session_retry_limit(100)
        .client().unwrap();

    let endpoint: EndpointDescription =
        ("opc.tcp://127.0.0.1:4855/", "None",
         MessageSecurityMode::None,
         UserTokenPolicy::anonymous())
        .into();

    let session = client.connect_to_endpoint(endpoint, IdentityToken::Anonymous).unwrap();

    if subscribe_to_values(session.clone()).is_ok() {
        let _ = Session::run(session);
    } else {
        println!("Error creating subscription");
    }
}

fn subscribe_to_values(session: Arc<RwLock<Session>>) -> Result<(), StatusCode> {
    let session = session.write().unwrap();

    // Create a subscription polling every 2s with a callback
    let subscription_id = session.create_subscription(50.0, 10, 100, 0, 0, true,
        DataChangeCallback::new(|changed_monitored_items| {
            println!("Data change from server:");
            changed_monitored_items.iter().for_each(|item| print_value(item));
        }))?;

    // Create some monitored items
    let items_to_create: Vec<MonitoredItemCreateRequest> = ["Test"].iter().map(|v| NodeId::new(2, *v).into()).collect();
    let create_result = session.create_monitored_items(subscription_id, TimestampsToReturn::Both, &items_to_create).unwrap();
    let _ = session.set_monitoring_mode(subscription_id, MonitoringMode::Reporting, &[create_result[0].monitored_item_id])?;

    Ok(())
}

fn print_value(item: &MonitoredItem) {
   let node_id = &item.item_to_monitor().node_id;
   let data_value = item.last_value();
   if let Some(ref value) = data_value.value {
       println!("Item \"{}\", Value = {:?}", node_id, value);
   } else {
       println!("Item \"{}\", Value not found, error: {}", node_id, data_value.status.as_ref().unwrap());
   }
}
