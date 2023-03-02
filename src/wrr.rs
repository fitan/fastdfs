// 加权轮询算法
pub struct WeightedRoundRobin {
    // 当前权重
    current_weight: usize,
    // 最大权重
    max_weight: usize,
    // 权重的最大公约数
    gcd_weight: usize,
    // 当前下标
    current_index: usize,
    // 服务列表
    servers: Vec<Server>,
}

pub struct Server {
    // 服务地址
    addr: String,
    // 权重
    weight: usize,
}

impl WeightedRoundRobin {
    pub fn new(servers: Vec<Server>) -> WeightedRoundRobin {
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

    pub fn next(&mut self) -> String {
        loop {
            self.current_index = (self.current_index + 1) % self.servers.len();
            if self.current_index == 0 {
                self.current_weight = self.current_weight - self.gcd_weight;
                if self.current_weight <= 0 {
                    self.current_weight = self.max_weight;
                    if self.current_weight == 0 {
                        return String::new();
                    }
                }
            }
            if self.servers[self.current_index].weight >= self.current_weight {
                return self.servers[self.current_index].addr.clone();
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

