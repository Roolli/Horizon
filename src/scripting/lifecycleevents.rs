pub enum LifeCycleEvent {
    Init = 0,
    OnResourceStart = 1,
    OnTick = 2,
    OnWindowEvent = 4,
    OnResourceStop = 8,
    BeforeRender = 16,
    AfterRender = 32,
    Cleanup = 64,
}
