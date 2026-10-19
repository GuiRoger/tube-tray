//! Detalhes específicos de sistema operacional.

/// `true` quando a barra de tarefas do Windows está no tema claro
/// (aí o ícone da bandeja precisa ser escuro). Fora do Windows, `false`.
#[cfg(windows)]
pub fn taskbar_is_light() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|k| k.get_value::<u32, _>("SystemUsesLightTheme"))
        .map(|v| v == 1)
        .unwrap_or(false)
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn taskbar_is_light() -> bool {
    false
}
