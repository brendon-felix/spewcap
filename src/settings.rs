
pub struct Settings<'a> {
    pub port: &'a str,
    pub baud_rate: u32,
    pub timestamps: bool,
    // pub log_folder: &str,
    // pub clear_on_start: bool,
}

impl Default for Settings<'_> {
    fn default() -> Self {
        Settings {
            port: "COM7",
            baud_rate: 115200,
            timestamps: false,
            // log_folder: r"C:\Users\felixb\OneDrive - HP Inc\Debugs\Springs",
            // clear_on_start: false,
        }
    }
}