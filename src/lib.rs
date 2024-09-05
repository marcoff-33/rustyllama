pub mod constants {
    pub const MODEL: &str = "llama3.1:latest";
    pub const DEFAULT_SM: &str = r#"
    Always Be Precise And Concise in your answers.
    
    Always end your answers with a wise and philosophical sentence.
    "#;
    pub const LATENCY_MS: f32 = 550.0;
    pub const SAMPLE_RATE: usize = 44100;
    pub const BUFFER_DURATION_SECONDS: usize = 30; // 30 seconds recording
    pub const CHANNELS: usize = 2;
    pub const BUFFER_SIZE: usize = SAMPLE_RATE * CHANNELS * 60; // 30 seconds buffer
}
