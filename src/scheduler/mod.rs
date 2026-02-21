#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub name: String,
    pub interval_ticks: u64,
    pub next_tick: u64,
    pub payload: String,
}

#[derive(Debug, Default)]
pub struct Scheduler {
    tick: u64,
    tasks: Vec<ScheduledTask>,
}

impl Scheduler {
    pub fn add_task(
        &mut self,
        name: impl Into<String>,
        interval_ticks: u64,
        payload: impl Into<String>,
    ) {
        let interval = interval_ticks.max(1);
        self.tasks.push(ScheduledTask {
            name: name.into(),
            interval_ticks: interval,
            next_tick: self.tick + interval,
            payload: payload.into(),
        });
    }

    pub fn heartbeat(&mut self) -> Vec<ScheduledTask> {
        self.tick += 1;
        let mut due = Vec::new();

        for task in &mut self.tasks {
            if self.tick >= task.next_tick {
                due.push(task.clone());
                task.next_tick = self.tick + task.interval_ticks;
            }
        }

        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_triggers_repeating_tasks() {
        let mut scheduler = Scheduler::default();
        scheduler.add_task("heartbeat", 2, "ping");

        assert!(scheduler.heartbeat().is_empty());
        let due = scheduler.heartbeat();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].payload, "ping");
    }
}
