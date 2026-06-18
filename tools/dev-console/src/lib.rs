#![deny(clippy::all)]

use std::collections::HashMap;

pub struct DevConsole {
    pub visible: bool,
    pub input_buffer: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub output_lines: Vec<ConsoleLine>,
    pub max_lines: usize,
    pub commands: HashMap<String, Box<dyn Fn(&[&str]) -> String>>,
}

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub text: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl DevConsole {
    pub fn new() -> Self {
        let mut console = Self {
            visible: false,
            input_buffer: String::new(),
            history: Vec::new(),
            history_index: 0,
            output_lines: Vec::new(),
            max_lines: 100,
            commands: HashMap::new(),
        };
        console.register_default_commands();
        console
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn submit(&mut self) {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() { return; }

        self.history.push(input.clone());
        self.history_index = self.history.len();
        self.input_buffer.clear();

        self.log(&format!("> {}", input), LogLevel::Debug);

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() { return; }

        let cmd = parts[0];
        let args = &parts[1..];

        if let Some(handler) = self.commands.get(cmd) {
            let result = handler(args);
            self.log(&result, LogLevel::Info);
        } else {
            self.log(&format!("Unknown command: {}. Type 'help' for commands.", cmd), LogLevel::Error);
        }
    }

    pub fn log(&mut self, text: &str, level: LogLevel) {
        self.output_lines.push(ConsoleLine { text: text.to_string(), level });
        if self.output_lines.len() > self.max_lines {
            self.output_lines.remove(0);
        }
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() { return; }
        if self.history_index > 0 {
            self.history_index -= 1;
        }
        self.input_buffer = self.history[self.history_index].clone();
    }

    pub fn history_down(&mut self) {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.input_buffer = self.history[self.history_index].clone();
        } else {
            self.history_index = self.history.len();
            self.input_buffer.clear();
        }
    }

    pub fn register_command<F>(&mut self, name: &str, handler: F)
    where
        F: 'static + Fn(&[&str]) -> String,
    {
        self.commands.insert(name.to_string(), Box::new(handler));
    }

    fn register_default_commands(&mut self) {
        self.register_command("help", |_| {
            "Available commands:\n  help      - Show this message\n  clear     - Clear console\n  entities  - List all entities\n  fps       - Show FPS\n  stats     - Show performance stats\n  spawn     - Spawn an entity: spawn <type>\n  tp        - Teleport: tp <x> <y> <z>\n  god       - Toggle god mode\n  gravity   - Set gravity: gravity <value>\n  time      - Set time scale: time <scale>\n  exit      - Exit console\n  echo      - Print text: echo <msg>\n  teleport  - Teleport: teleport <x> <y> <z>\n  give      - Give item: give <item> [amount]\n  kill      - Kill entity: kill <entity_id>\n  heal      - Heal player: heal <amount>\n  set       - Set variable: set <var> <val>\n  get       - Get variable: get <var>"
                .to_string()
        });

        self.register_command("clear", |_| { "".to_string() });
        self.register_command("entities", |_| { "Entity list: (stub)".to_string() });
        self.register_command("fps", |_| { "FPS: 60.0 (stub)".to_string() });
        self.register_command("stats", |_| {
            "=== Performance Stats ===\nFrame time: 16.6ms\nDraw calls: 0\nEntities: 0\nTriangles: 0"
                .to_string()
        });

        self.register_command("god", |_| { "God mode: ON".to_string() });

        self.register_command("spawn", |args| {
            if args.is_empty() { return "Usage: spawn <type>".to_string(); }
            format!("Spawned: {}", args.join(" "))
        });

        self.register_command("tp", |args| {
            if args.len() < 3 { return "Usage: tp <x> <y> <z>".to_string(); }
            format!("Teleported to ({}, {}, {})", args[0], args[1], args[2])
        });

        self.register_command("gravity", |args| {
            if args.is_empty() { return "Usage: gravity <value>".to_string(); }
            format!("Gravity set to: {}", args[0])
        });

        self.register_command("time", |args| {
            if args.is_empty() { return "Usage: time <scale>".to_string(); }
            format!("Time scale set to: {}", args[0])
        });

        self.register_command("exit", |_| { "Exiting console...".to_string() });
        self.register_command("echo", |args| format!("{}", args.join(" ")));
        self.register_command("teleport", |args| {
            if args.len() < 3 { return "Usage: teleport <x> <y> <z>".to_string(); }
            format!("Teleported to ({}, {}, {})", args[0], args[1], args[2])
        });
        self.register_command("give", |args| {
            if args.is_empty() { return "Usage: give <item> [amount]".to_string(); }
            format!("Gave {} x {}", args.get(1).unwrap_or(&"1"), args[0])
        });
        self.register_command("kill", |args| {
            if args.is_empty() { return "Usage: kill <entity_id>".to_string(); }
            format!("Killed entity {}", args[0])
        });
        self.register_command("heal", |args| {
            if args.is_empty() { return "Usage: heal <amount>".to_string(); }
            format!("Healed for {}", args[0])
        });
        self.register_command("set", |args| {
            if args.len() < 2 { return "Usage: set <variable> <value>".to_string(); }
            format!("Set {} = {}", args[0], args[1])
        });
        self.register_command("get", |args| {
            if args.is_empty() { return "Usage: get <variable>".to_string(); }
            format!("{} = ?", args[0])
        });
    }

    pub fn clear_log(&mut self) {
        self.output_lines.clear();
    }

    pub fn execute(&mut self, command: &str) -> String {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return String::new(); }
        match self.commands.get(parts[0]) {
            Some(handler) => handler(&parts[1..]),
            None => format!("Unknown command: {}", parts[0]),
        }
    }

    pub fn render_text(&self) -> String {
        let mut output = String::new();
        for line in &self.output_lines {
            let prefix = match line.level {
                LogLevel::Info => "",
                LogLevel::Warning => "[WARN] ",
                LogLevel::Error => "[ERR] ",
                LogLevel::Debug => "",
            };
            output.push_str(&format!("{}{}\n", prefix, line.text));
        }
        output
    }
}

impl Default for DevConsole {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_creation() {
        let console = DevConsole::new();
        assert!(!console.visible);
    }

    #[test]
    fn test_toggle() {
        let mut console = DevConsole::new();
        console.toggle();
        assert!(console.visible);
        console.toggle();
        assert!(!console.visible);
    }

    #[test]
    fn test_submit_command() {
        let mut console = DevConsole::new();
        console.input_buffer = "help".to_string();
        console.submit();
        assert!(!console.output_lines.is_empty());
    }

    #[test]
    fn test_execute_help() {
        let mut console = DevConsole::new();
        let result = console.execute("help");
        assert!(result.contains("spawn"));
        assert!(result.contains("tp"));
    }

    #[test]
    fn test_execute_spawn() {
        let mut console = DevConsole::new();
        let result = console.execute("spawn enemy");
        assert_eq!(result, "Spawned: enemy");
    }

    #[test]
    fn test_execute_teleport() {
        let mut console = DevConsole::new();
        let result = console.execute("tp 10 20 30");
        assert_eq!(result, "Teleported to (10, 20, 30)");
    }

    #[test]
    fn test_execute_unknown() {
        let mut console = DevConsole::new();
        let result = console.execute("nonexistent");
        assert!(result.contains("Unknown command"));
    }

    #[test]
    fn test_log() {
        let mut console = DevConsole::new();
        console.log("Test message", LogLevel::Info);
        assert_eq!(console.output_lines.len(), 1);
    }

    #[test]
    fn test_history() {
        let mut console = DevConsole::new();
        console.input_buffer = "help".to_string();
        console.submit();
        assert_eq!(console.history.len(), 1);
        console.history_up();
        assert_eq!(console.input_buffer, "help");
    }

    #[test]
    fn test_custom_command() {
        let mut console = DevConsole::new();
        console.register_command("hello", |args| {
            format!("Hello, {}!", args.first().unwrap_or(&"world"))
        });
        let result = console.execute("hello agent");
        assert_eq!(result, "Hello, agent!");
    }

    #[test]
    fn test_clear_log() {
        let mut console = DevConsole::new();
        console.log("test", LogLevel::Info);
        assert_eq!(console.output_lines.len(), 1);
        console.clear_log();
        assert!(console.output_lines.is_empty());
    }

    #[test]
    fn test_toggle_console() {
        let mut console = DevConsole::new();
        assert!(!console.visible);
        console.toggle();
        assert!(console.visible);
        console.toggle();
        assert!(!console.visible);
    }

    #[test]
    fn test_render_text() {
        let mut console = DevConsole::new();
        console.log("info msg", LogLevel::Info);
        console.log("warn msg", LogLevel::Warning);
        let text = console.render_text();
        assert!(text.contains("info msg"));
        assert!(text.contains("[WARN] warn msg"));
    }
}
