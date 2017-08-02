extern crate crypto;
use std::collections::{HashMap};
use self::crypto::md5::Md5;
use self::crypto::digest::Digest;
use std::collections::LinkedList;



#[derive(Debug, Deserialize)]
struct NodeConfig {
    ip: String,
    port: i32,
}

#[derive(Debug, Deserialize)]
struct ZoneConfig {
    nodes: Vec<NodeConfig>,
}

#[derive(Debug, Deserialize)]
struct RingConfig {
    zones: Vec<ZoneConfig>,
    part_power: usize,
}

struct Node  {
    ip: String,
    port:i32,
    zone_id: usize,
}

impl Clone for Node {
    fn clone(&self) -> Node {
        Node {
            ip: self.ip.clone(),
            port: self.port,
            zone_id: self.zone_id,
        }
    }
}

struct Ring {
    part_replica_node: Vec<Vec<usize>>,
    nodes: HashMap<usize, Node>,
    replicas_count: usize,
    parts_count: usize,
    zone_nodes: Vec<Vec<usize>>,

}

fn cal_power(value: usize, mut times: usize ) -> usize {
    let mut res = 1;
    while times > 0 {
        res = res * value;
        times -= 1;
    }

    res
}

impl Ring {
    fn build(config: RingConfig) -> Result<Ring, ()> {
        let replicas_count = config.zones.len();
        let mut hash_nodes = HashMap::new();
        let mut index = 0;
        let mut zone_nodes = Vec::new();
        let mut zone_id = 0;
        for zone_config in &config.zones {
            let mut nodes : Vec<usize>= Vec::new();
            for node_config in &zone_config.nodes{
                let node = Node {
                    ip: node_config.ip.clone(),
                    port: node_config.port,
                    zone_id: zone_id,
                };
                hash_nodes.insert(index, node);
                nodes.push(index);
                index += 1;
            }

            zone_id += 1;
            zone_nodes.push(nodes);
        }

        let mut part_replica_node = Vec::new();
        for replica_idx in 0..replicas_count {
            part_replica_node.push(Vec::new());
        }
        let parts_count = cal_power(2, config.part_power);
        for replica_idx in 0..replicas_count {
            let mut array_idx = 0;
            let zone_len = (zone_nodes[replica_idx]).len();
            for part_id in 0..parts_count {
                let node_id = zone_nodes[replica_idx][array_idx];
                part_replica_node[replica_idx].push(node_id);
                array_idx = (array_idx + 1) % zone_len;
            }
        }

        let ring =  Ring {
            part_replica_node: part_replica_node,
            nodes: hash_nodes,
            replicas_count: replicas_count,
            parts_count: parts_count,
            zone_nodes: zone_nodes,
        };
        Result::Ok(ring)
    }

    fn get_replicas(&self, hash_code: usize) -> Vec<Node>{
        let mut res = Vec::new();
        let replicas_count = self.replicas_count;
        let hash_idx = hash_code % self.parts_count;
        for replica_id in 0..replicas_count {
            let node_idx = self.part_replica_node[replica_id][hash_idx];
            let in_node = self.nodes.get(&node_idx).unwrap();
            let out_node = in_node.clone();
            res.push(out_node);
        }
        res
    }

}
