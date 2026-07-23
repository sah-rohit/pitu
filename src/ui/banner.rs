use console::{style, Term};
use std::thread;

/// Render large 3D retro-modern block ASCII title banner for PITU
pub fn print_header_banner() {
    let logo_lines = vec![
        r"  ██████╗ ██╗████████╗██╗   ██╗",
        r"  ██╔══██╗██║╚══██╔══╝██║   ██║",
        r"  ██████╔╝██║   ██║   ██║   ██║",
        r"  ██╔═══╝ ██║   ██║   ██║   ██║",
        r"  ██║     ██║   ██║   ╚██████╔╝",
        r"  ╚═╝     ╚═╝   ╚═╝    ╚═════╝ ",
    ];

    println!();
    for line in logo_lines {
        println!("{}", style(line).bold().cyan());
    }
    println!(
        "  {} {}",
        style("PITU WORKBENCH").bold().yellow(),
        style("v0.1.0 • Scriptable CLI Image Engine").dim()
    );
    println!();
}

/// Render Claude Code style Dashboard Card with telemetry and recent stats
pub fn print_welcome_dashboard() {
    let term_width = Term::stdout().size().1 as usize;
    let card_width = if term_width > 80 { 77 } else { term_width.saturating_sub(4).max(40) };

    let num_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let top_border = format!("  ╭{}╮", "─".repeat(card_width - 2));
    let bot_border = format!("  ╰{}╯", "─".repeat(card_width - 2));

    println!("{}", style(top_border).cyan());
    println!(
        "  │  {}│",
        fit_line_content(&format!("Pitu Workbench v0.1.0 Engine"), card_width - 5)
    );
    println!(
        "  │  {}│",
        fit_line_content("────────────────────────────", card_width - 5)
    );
    println!(
        "  │  🧠 {}│",
        fit_line_content(
            &format!("{}: Sobel Edge + Shannon 2D Entropy", style("Smart Engine").cyan().bold()),
            card_width - 9
        )
    );
    println!(
        "  │  ⚡ {}│",
        fit_line_content(
            &format!("{}: Rayon Threadpool ({} CPU Cores)", style("Performance ").green().bold(), num_cores),
            card_width - 9
        )
    );
    println!(
        "  │  🖼️ {}│",
        fit_line_content(
            &format!("{}: PNG, JPEG, WebP, GIF, BMP, TIFF, ICO", style("Codecs      ").magenta().bold()),
            card_width - 9
        )
    );
    println!(
        "  │  📁 {}│",
        fit_line_content(
            &format!("{}: Auto-sanitized Drag & Drop or Pasted Paths", style("Input Mode  ").blue().bold()),
            card_width - 9
        )
    );
    println!("{}", style(bot_border).cyan());
    println!();
}

fn fit_line_content(content: &str, target_width: usize) -> String {
    let plain = console::strip_ansi_codes(content);
    let len = plain.chars().count();
    if len >= target_width {
        content.chars().take(target_width).collect()
    } else {
        format!("{}{}", content, " ".repeat(target_width - len))
    }
}

/// Print status footer with keybindings
pub fn print_footer_hints() {
    println!(
        "  {}",
        style("Nav: [↑/↓] Select  [Enter] Confirm  [Tab] Next  [q/Esc] Quit").dim()
    );
    println!();
}
