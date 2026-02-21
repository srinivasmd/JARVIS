use std::time::{Duration, Instant};

use crate::{
    core::Agent,
    policy::{Permission, Policy},
    providers::{EchoProvider, ProviderRouter},
    sandbox::DenyByDefaultSandbox,
};

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub iterations: usize,
    pub total: Duration,
    pub average_micros: u128,
}

pub fn run_startup_benchmark(iterations: usize) -> BenchmarkResult {
    let loops = iterations.max(1);
    let start = Instant::now();

    for _ in 0..loops {
        let policy = Policy::allow_list([
            Permission::MemoryRead,
            Permission::MemoryWrite,
            Permission::ToolExec,
        ]);
        let router = ProviderRouter::new(vec![Box::new(EchoProvider)]);
        let mut agent = Agent::new(policy, router, Box::new(DenyByDefaultSandbox));
        let _ = agent.run_prompt("bench");
    }

    let total = start.elapsed();
    BenchmarkResult {
        iterations: loops,
        average_micros: total.as_micros() / loops as u128,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_returns_non_zero_iterations() {
        let result = run_startup_benchmark(10);
        assert_eq!(result.iterations, 10);
        assert!(result.total.as_nanos() > 0);
    }
}
