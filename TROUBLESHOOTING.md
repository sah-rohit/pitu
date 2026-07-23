# `pitu` Troubleshooting, Prerequisites & OS Compatibility Guide 📷⚡

This guide covers system prerequisites, terminal configuration tips, and step-by-step solutions for **macOS**, **Windows**, and **Linux**.

---

## 📋 System Prerequisites

Before building or running `pitu`, ensure your operating system has the necessary build tools and dependencies installed:

### 🍎 macOS Prerequisites
1. **Xcode Command Line Tools**:
   Required for Rust C++ bindings and native terminal tools.
   ```bash
   xcode-select --install
   ```
2. **Rust Toolchain**:
   Install via rustup:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

---

### 🪟 Windows Prerequisites
1. **Build Tools for Visual Studio**:
   Install Visual Studio C++ Build Tools (or the GNU toolchain) via [rustup.rs](https://rustup.rs).
2. **PowerShell 5.1+ or Windows Terminal**:
   Recommended for ANSI truecolor rendering.

---

### 🐧 Linux Prerequisites
1. **C Compiler & Build Tools**:
   - **Ubuntu/Debian**: `sudo apt update && sudo apt install build-essential pkg-config xdg-utils`
   - **Fedora/RHEL**: `sudo dnf groupinstall "Development Tools" && sudo dnf install xdg-utils`
   - **Arch Linux**: `sudo pacman -S base-devel xdg-utils`

---

## 💻 OS-Specific Installation & Setup Guides

### 🍎 macOS Setup (Terminal / iTerm2)

#### One-Liner Installer
```bash
chmod +x install.sh && ./install.sh
```

#### PATH Configuration (`.zshrc` / `.bash_profile`)
If `pitu` is installed to `~/.local/bin`, ensure your shell path includes it:
```bash
echo 'export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

#### Native macOS GUI Folder Opener
`pitu` automatically uses macOS's native `open` command to launch Finder when clicking **`📁 Open Folder in Desktop File Manager`** after saving an image!

---

### 🪟 Windows Setup (PowerShell / Command Prompt)

#### Recommended: Automated PowerShell Installer
Open PowerShell and run:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; .\install.ps1
```

#### Adding `pitu` to Windows `%PATH%` (If command not found)
1. Press `Win + R`, type `sysdm.cpl`, and press **Enter**.
2. Go to **Advanced** tab ➔ Click **Environment Variables**.
3. Under **User variables**, select `Path` ➔ Click **Edit**.
4. Click **New** and add:
   - `%USERPROFILE%\.local\bin`
   - `%USERPROFILE%\.cargo\bin`
5. Click **OK** and restart your PowerShell or Command Prompt.

---

### 🐧 Linux Setup (Ubuntu / Fedora / Arch)

#### Installation
```bash
chmod +x install.sh && ./install.sh
```

---

## 🛠️ Common Errors & How to Fix Them

### ❌ Error 1: `'pitu' is not recognized as an internal or external command` (Windows)
- **Cause**: The folder containing `pitu.exe` (`%USERPROFILE%\.local\bin` or `%USERPROFILE%\.cargo\bin`) is not in your Windows `%PATH%`.
- **Fix**:
  1. Open PowerShell as Administrator.
  2. Run:
     ```powershell
     [Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";$env:USERPROFILE\.local\bin", "User")
     ```
  3. Close and reopen your terminal window.

---

### ❌ Error 2: `xcrun: error: invalid active developer path` (macOS)
- **Cause**: Xcode Command Line Tools were removed or unlinked after a macOS software update.
- **Fix**: Re-install Xcode Command Line Tools:
  ```bash
  xcode-select --install
  ```

---

### ❌ Error 3: Terminal Colors or Box Characters Distorted in Windows Command Prompt
- **Cause**: Legacy `cmd.exe` does not enable ANSI truecolor VT processing by default.
- **Fix**:
  - **Recommended**: Use **Windows Terminal** (available free from Microsoft Store) or PowerShell.
  - Or enable VT100 colors in legacy `cmd.exe` by running:
    ```cmd
    reg add HKCU\Console /v VirtualTerminalLevel /t REG_DWORD /d 1 /f
    ```

---

### ❌ Error 4: File Path Escape Backslashes (`C:\Users\Name\Pictures`)
- **Cause**: Windows backslashes `\` when pasted into terminal prompts.
- **Fix**: `pitu` automatically sanitizes backslashes, double quotes (`"`), single quotes (`'`), and `file:///` URLs. You can safely paste Windows paths directly!

---

### ❌ Error 5: `ExecutionPolicy` Blocking PowerShell Installer Script
- **Cause**: Windows restricts unsigned PowerShell scripts by default.
- **Fix**:
  Run PowerShell with process-level bypass:
  ```powershell
  Set-ExecutionPolicy Bypass -Scope Process -Force; .\install.ps1
  ```
