pub trait Device {
    fn test(&self) -> bool;

    fn read(&mut self) -> u8;

    fn write(&mut self, value: u8);
}
