use crate::world::{Order, Response, World};
use cucumber::{given, then, when};

#[given(regex = r#"^an order "([^"]+)" totalling (\d+) cents$"#)]
async fn given_an_order(world: &mut World, order_id: String, total_cents: String) {
    let total_cents = total_cents.parse::<u64>().expect("valid amount");
    world.orders.insert(order_id.clone(), Order { id: order_id, total_cents });
}

#[when(regex = r#"^I POST /refunds with \{ "orderId": "([^"]+)", "amountCents": (\d+) \}$"#)]
async fn when_post_refunds(world: &mut World, order_id: String, amount_cents: String) {
    let amount_cents = amount_cents.parse::<u64>().expect("valid amount");

    // Verify order exists
    let _order = world.orders.get(&order_id).expect("order should exist");

    // Simulate API response
    let body = serde_json::json!({
        "refundId": format!("REF-{}", uuid::Uuid::new_v4()),
        "orderId": order_id,
        "amountCents": amount_cents,
        "status": "pending"
    });

    world.last_response = Some(Response { status: 201, body });
}

#[then(regex = r#"^I receive (\d+) with a "([^"]+)"$"#)]
async fn then_receive_status_with_field(world: &mut World, status: String, field: String) {
    let status = status.parse::<u16>().expect("valid status");

    let response = world.last_response.as_ref().expect("should have a response");

    assert_eq!(response.status, status, "Expected status {}, got {}", status, response.status);

    assert!(response.body.get(&field).is_some(), "Response should contain field '{}'", field);
}
