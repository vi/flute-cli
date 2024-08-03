pub trait ExtractIoErrorFromFlute<T> {
    fn ee(self) -> Result<T, std::io::Error>;
}

impl<T> ExtractIoErrorFromFlute<T> for Result<T, flute::error::FluteError> {
    fn ee(self) -> Result<T, std::io::Error> {
        self.map_err(|x|x.0)
    }
}
