// 加权轮询算法
pub struct WeightedRoundRobin<T: Weight> {
    // 当前权重
    current_weight: usize,
    // 最大权重
    max_weight: usize,
    // 权重的最大公约数
    gcd_weight: usize,
    // 当前下标
    current_index: usize,
    // 服务列表
    servers: Vec<T>,
}

pub trait Weight {
    fn weight(&self) -> usize;
}

impl<T: Weight> WeightedRoundRobin<T> {
    pub fn new(servers: Vec<T>) -> WeightedRoundRobin<T> {
        let mut max_weight = 0;
        let mut gcd_weight = 0;
        for server in &servers {
            if server.weight > max_weight {
                max_weight = server.weight;
            }
            gcd_weight = gcd(gcd_weight, server.weight);
        }
        WeightedRoundRobin {
            current_weight: 0,
            max_weight,
            gcd_weight,
            current_index: 0,
            servers,
        }
    }

    pub fn next(&mut self) -> Option<T> {
        loop {
            self.current_index = (self.current_index + 1) % self.servers.len();
            if self.current_index == 0 {
                self.current_weight = self.current_weight - self.gcd_weight;
                if self.current_weight <= 0 {
                    self.current_weight = self.max_weight;
                    if self.current_weight == 0 {
                        return None
                    }
                }
            }
            if self.servers[self.current_index].weight >= self.current_weight {
                return Some(self.servers[self.current_index])
            }
        }
    }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

