
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    // /// Port name (eg. COM23 OR /dev/ttyUSB0)
    // #[arg(short, long)]
    // port: Option<String>,

    // /// Baud rate
    // #[arg(short, long)]
    // baud_rate: Option<u32>,

    /// Prepend timestamps in log
    #[arg(short, long)]
    pub timestamps: bool,

    /// Folder path to save logs
    #[arg(short, long)]
    pub logging: bool,

    // /// Folder path to save logs
    // #[arg(short, long)]
    // log_folder: Option<String>,

    #[arg(long)]
    pub list: bool
}

// impl Args {
    
// }

// pub fn args_handler() -> Result<()> {
//     let args = Args::parse();
//     if args.list {
//         list_ports()?;
//     }
//     Ok(())
// }

