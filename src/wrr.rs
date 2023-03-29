use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc};
use tokio::sync::Mutex;

// 加权轮询算法
pub struct WeightedRoundRobin<T: Weight> {
    // 当前权重
    pub current_weight: i32,
    // 最大权重
    pub max_weight: i32,
    // 权重的最大公约数
    pub gcd_weight: i32,
    // 当前下标
    pub current_index: Arc<Mutex<i32>>,
    // 服务列表
    pub servers: Vec<Arc<Mutex<T>>>,
}

pub trait Weight {
    fn weight(&self) -> i32;
}

impl<T: Weight + Send> WeightedRoundRobin<T> {
    pub async fn new(servers: Vec<Arc<Mutex<T>>>) -> WeightedRoundRobin<T> {
        let mut max_weight = 0;
        let mut gcd_weight = 0;
        for server in &servers {
            let weight = server.clone().lock().await.weight();
            if weight > max_weight {
                max_weight = weight;
            }
            gcd_weight = gcd(gcd_weight, weight);
        }
        WeightedRoundRobin {
            current_weight: 0,
            max_weight,
            gcd_weight,
            current_index: Arc::new(Mutex::new(0)),
            servers,
        }
    }
    
    pub async fn next(&mut self) -> Option<Arc<Mutex<T>>> {
        loop {
            let mut current_index = self.current_index.lock().await;
            *current_index = (*current_index + 1) % (self.servers.len() as i32);
            if *current_index == 0 {
                self.current_weight = self.current_weight - self.gcd_weight;
                if self.current_weight <= 0 {
                    self.current_weight = self.max_weight;
                    if self.current_weight == 0 {
                        return None
                    }
                }
            }
            if self.servers[*current_index as usize].clone().lock().await.weight() >= self.current_weight {
                return Some(self.servers[*current_index as usize].clone())
            }
        }
    }
}

fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

