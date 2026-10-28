use std::{io::{BufRead, BufReader}, path::{Path, PathBuf}, process::{Child, Command, Stdio}, sync::mpsc::{self, Receiver, Sender}, thread, time::Instant};

#[derive(Debug, Clone)]
pub enum RunEvent { Output(String), Error(String), Finished(i32, f32) }

pub struct Runner {
    child: Option<Child>,
    rx: Option<Receiver<RunEvent>>,
    started: Option<Instant>,
    pub running: bool,
}
impl Default for Runner { fn default() -> Self { Self { child: None, rx: None, started: None, running: false } } }

fn command_for(path: &Path, root: &Path) -> Result<Command, String> {
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
    let mut c = match ext.as_str() {
        "py" => { let mut c = Command::new("python"); c.arg(path); c }
        "js" | "mjs" | "cjs" => { let mut c = Command::new("node"); c.arg(path); c }
        "rs" => {
            let mut c = Command::new("cargo");
            if root.join("Cargo.toml").exists() { c.arg("run").arg("--quiet").current_dir(root); }
            else { return Err("Rust run requires a Cargo project with Cargo.toml in the workspace. Create a Cargo project first.".into()); }
            c
        }
        _ => return Err(format!("Run is not configured for .{ext} files. Supported in this release: Python (.py), Node.js (.js/.mjs/.cjs), and Rust Cargo projects (.rs).")),
    };
    if !c.get_current_dir().is_some() { c.current_dir(root); }
    Ok(c)
}
impl Runner {
    pub fn start(&mut self, path: &Path, root: &Path) -> Result<(), String> {
        self.stop();
        let mut cmd = command_for(path, root)?;
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| format!("Could not start program: {e}. Check that the required runtime is installed and on PATH."))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let (tx, rx): (Sender<RunEvent>, Receiver<RunEvent>) = mpsc::channel();
        if let Some(out) = stdout {
            let tx = tx.clone();
            thread::spawn(move || for line in BufReader::new(out).lines() { match line { Ok(s) => { let _ = tx.send(RunEvent::Output(s)); }, Err(e) => { let _ = tx.send(RunEvent::Error(e.to_string())); break; } } });
        }
        if let Some(err) = stderr {
            let tx = tx.clone();
            thread::spawn(move || for line in BufReader::new(err).lines() { match line { Ok(s) => { let _ = tx.send(RunEvent::Error(s)); }, Err(e) => { let _ = tx.send(RunEvent::Error(e.to_string())); break; } } });
        }
        self.child = Some(child);
        self.rx = Some(rx);
        self.started = Some(Instant::now());
        self.running = true;
        Ok(())
    }
    pub fn stop(&mut self) -> bool {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.running = false;
            true
        } else { false }
    }
    pub fn poll(&mut self) -> Vec<RunEvent> {
        let mut events = Vec::new();
        if let Some(rx) = &self.rx { while let Ok(e) = rx.try_recv() { events.push(e); } }
        if self.running {
            if let Some(child) = &mut self.child {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let elapsed = self.started.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);
                        events.push(RunEvent::Finished(status.code().unwrap_or(-1), elapsed));
                        self.child = None; self.running = false;
                    }
                    Ok(None) => {}
                    Err(e) => { events.push(RunEvent::Error(format!("Process status error: {e}"))); self.child = None; self.running = false; }
                }
            }
        }
        events
    }
    pub fn root_from_path(path: &PathBuf) -> PathBuf { path.parent().unwrap_or(Path::new(".")).to_path_buf() }
}
