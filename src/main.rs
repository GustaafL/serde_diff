use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    metadata: Metadata,
    pub spec: Spec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    labels: HashMap<String, String>,
    expires: String,
    id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub hostname: String,
}

fn get_keys() {
    let data = r#"
[
  {
    "metadata": {
      "name": "161bee39-cf07-4e31-90ba-6593c9f505cb",
      "labels": {
        "application": "api",
        "owner": "team_x"
      },
      "expires": "2021-12-06T20:49:04.136656523Z",
      "id": 1638823144137190452
    },
    "spec": {
      "hostname": "host1.example.com"
    }
  },
  {
    "metadata": {
      "name": "c1b3ee09-8e4a-49d4-93b8-95cbcb676f20",
      "labels": {
        "application": "database",
        "owner": "team_y"
      },
      "expires": "2021-12-06T20:49:55.23841272Z",
      "id": 1638823195247684748
    },
    "spec": {
      "hostname": "host2.example.com"
    }
  }
]
    "#;

    let value: serde_json::Value = serde_json::from_str(data).unwrap();
    for key in value.as_object().unwrap().keys() {
        println!("{}", key);
    }

}


fn main() -> Result<()> {
    let json = r#"
[
  {
    "metadata": {
      "name": "161bee39-cf07-4e31-90ba-6593c9f505cb",
      "labels": {
        "application": "api",
        "owner": "team_x"
      },
      "expires": "2021-12-06T20:49:04.136656523Z",
      "id": 1638823144137190452
    },
    "spec": {
      "hostname": "host1.example.com"
    }
  },
  {
    "metadata": {
      "name": "c1b3ee09-8e4a-49d4-93b8-95cbcb676f20",
      "labels": {
        "application": "database",
        "owner": "team_y"
      },
      "expires": "2021-12-06T20:49:55.23841272Z",
      "id": 1638823195247684748
    },
    "spec": {
      "hostname": "host2.example.com"
    }
  }
]
    "#;
    get_keys();
    let nodes: Vec<Node> = serde_json::from_str(json)?;
    println!("{:?}", nodes);
    Ok(())
}
