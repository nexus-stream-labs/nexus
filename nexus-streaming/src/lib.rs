
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod windowing;
pub mod joins;
pub mod aggregation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub key: Option<String>,
    pub value: Vec<u8>,
    pub timestamp: u64,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Window {
    pub start: Instant,
    pub duration: Duration,
    pub messages: Vec<StreamMessage>,
}

pub trait StreamProcessor {
    async fn process(&mut self, message: StreamMessage) -> anyhow::Result<Vec<StreamMessage>>;
    async fn flush(&mut self) -> anyhow::Result<Vec<StreamMessage>>;
}

#[derive(Debug)]
pub struct WindowedProcessor {
    window_duration: Duration,
    current_window: Option<Window>,
}

impl WindowedProcessor {
    pub fn new(window_duration: Duration) -> Self {
        Self {
            window_duration,
            current_window: None,
        }
    }
    
    fn should_create_new_window(&self) -> bool {
        match &self.current_window {
            None => true,
            Some(window) => window.start.elapsed() >= self.window_duration,
        }
    }
}

impl StreamProcessor for WindowedProcessor {
    async fn process(&mut self, message: StreamMessage) -> anyhow::Result<Vec<StreamMessage>> {
        if self.should_create_new_window() {
            let old_window = self.current_window.take();
            self.current_window = Some(Window {
                start: Instant::now(),
                duration: self.window_duration,
                messages: Vec::new(),
            });
            
            if let Some(window) = old_window {
                return Ok(window.messages);
            }
        }
        
        if let Some(ref mut window) = self.current_window {
            window.messages.push(message);
        }
        
        Ok(Vec::new())
    }
    
    async fn flush(&mut self) -> anyhow::Result<Vec<StreamMessage>> {
        if let Some(window) = self.current_window.take() {
            Ok(window.messages)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_windowed_processor() {
        let mut processor = WindowedProcessor::new(Duration::from_millis(100));
        
        let message = StreamMessage {
            key: Some("test".to_string()),
            value: b"test_value".to_vec(),
            timestamp: 1640995200000,
            headers: HashMap::new(),
        };
        
        let result = processor.process(message).await.unwrap();
        assert!(result.is_empty()); // First message in window
    }
}
