use reqwest::Client;
use serde_json::Value;

async fn send_graphql(gateway: &str, query: Value) -> Option<Value> {
    let client = Client::new();

    let res = client
        .post(format!("{}/{}", gateway, "graphql"))
        .header("Content-Type", "application/json")
        .json(&query)
        .send()
        .await
        .ok()?;

    let json_res: Option<Value> = res.json().await.ok();

    json_res
}

async fn get_price_storage_hash() -> Option<String> {
    let query = serde_json::json!({
        "variables": {},
        "query": format!(r#"
        query {{
            transactions(
                sort: HEIGHT_DESC,
                tags: [
                {{
                    name: "type",
                    values: ["redstone-oracles"]
                }},
                {{
                    name: "dataFeedId",
                    values: ["AR"]
                }},
                {{
                    name: "dataServiceId",
                    values: ["redstone-primary-prod"]
                }}
                ],
                owners: ["I-5rWUehEv-MjdK9gFw09RxfSLQX9DIHxG614Wf8qo0"]
            ) {{
                edges {{
                    node {{
                        id
                        tags {{
                            name
                            value
                        }}
                        owner {{
                            address
                        }}
                    }}
                }}
            }}
        }}
        "#)
    });

    let res = send_graphql("https://arweave.net", query).await.unwrap();
    let transaction = res
        .get("data")
        .and_then(|data| data.get("transactions"))
        .and_then(|transactions| transactions.get("edges"))
        .and_then(|edges| edges.get(0))
        .and_then(|edge| edge.get("node"));
    let mut id = None;
    if let Some(tx) = transaction {
        if let Some(txid) = tx.get("id").and_then(|id_value| id_value.as_str()) {
            id = Some(String::from(txid));
        }
    }
    return id;
}

pub async fn fetch_price_onchain() -> Option<f64> {
    let price_hash = get_price_storage_hash().await;

    if price_hash.is_some() {
        let req = reqwest::get(format!("https://arweave.net/{}", price_hash.unwrap()))
            .await
            .ok()?;
        let json: Value = req.json::<Value>().await.ok()?;
        let ar_value = json
            .get("dataPoints")
            .and_then(|data_points| data_points.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first_point| first_point.get("value"))
            .and_then(|value| value.as_f64());

        ar_value
    } else {
        None
    }
}
