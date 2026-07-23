# `pitu` Troubleshooting & OS Compatibility Guide 📷⚡

This guide covers common setup questions, terminal configuration tips, and step-by-step solutions for **Windows**, **macOS**, and **Linux**.

---

## 💻 OS-Specific Installation & Setup Guides

### 🪟 Windows Setup (PowerShell / Command Prompt)

#### Recommended: Automated PowerShell Installer
Open PowerShell and run:
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; .\install.ps1
```

#### Manual Cargo Installation
```cmd
cargo install --path .
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

---

### 🐧 Linux Setup (Ubuntu / Fedora / Arch)

#### Installation
```bash
chmod +x install.sh && ./install.sh
```

#### Desktop GUI File Opener Dependency
`pitu` uses `xdg-open` to launch the Desktop File Manager. Ensure `xdg-utils` is installed:
- **Ubuntu/Debian**: `sudo apt install xdg-utils`
- **Fedora/RHEL**: `sudo dnf install xdg-utils`
- **Arch Linux**: `sudo pacman -S xdg-utils`

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

### ❌ Error 2: Terminal Colors or Box Characters Distorted in Windows Command Prompt
- **Cause**: Legacy `cmd.exe` does not enable ANSI truecolor VT processing by default.
- **Fix**:
  - **Recommended**: Use **Windows Terminal** (available free from Microsoft Store) or PowerShell.
  - Or enable VT100 colors in legacy `cmd.exe` by running:
    ```cmd
    reg add HKCU\Console /v VirtualTerminalLevel /t REG_DWORD /d 1 /f
    ```

---

### ❌ Error 3: File Path Escape Backslashes (`C:\Users\Name\Pictures`)
- **Cause**: Windows backslashes `\` when pasted into terminal prompts.
- **Fix**: `pitu` automatically sanitizes backslashes, double quotes (`"`), single quotes (`'`), and `file:///` URLs. You can safely paste Windows paths directly!

---

### ❌ Error 4: `ExecutionPolicy` Blocking PowerShell Installer Script
- **Cause**: Windows restricts unsigned PowerShell scripts by default.
- **Fix**:
  Run PowerShell with process-level bypass:
  ```powershell
  Set-ExecutionPolicy Bypass -Scope Process -Force; .\install.ps1
  ```

---

### ❌ Error 5: `Permission Denied` When Saving Files
- **Cause**: Trying to save output files directly into protected directories (like `C:\Program Files` or `/usr/bin`).
- **Fix**: Specify your user home directory or pictures folder (e.g. `./output` or `C:\Users\YourName\Pictures`).
