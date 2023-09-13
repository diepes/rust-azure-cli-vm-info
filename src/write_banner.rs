use colored::Colorize;
fn _write_banner() {
    // https://manytools.org/hacker-tools/ascii-banner/
    let banner = r#"
    ████████╗██╗    ██╗ ██████╗ 
    ╚══██╔══╝██║    ██║██╔════╝ 
       ██║   ██║ █╗ ██║██║  ███╗
       ██║   ██║███╗██║██║   ██║
       ██║   ╚███╔███╔╝╚██████╔╝
       ╚═╝    ╚══╝╚══╝  ╚═════╝ 
                                
"#;
    println!("{}", banner.red());
}
