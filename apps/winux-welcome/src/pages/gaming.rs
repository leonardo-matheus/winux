// Winux Welcome - Gaming Setup
// Optional page for setting up gaming environment

use gtk4::prelude::*;
use gtk4::{Box, CheckButton, Label, Orientation};
use libadwaita as adw;
use adw::prelude::*;
use adw::{PreferencesGroup, PreferencesPage, ActionRow, SwitchRow};
use std::cell::RefCell;
use std::rc::Rc;

use crate::window::WelcomeState;

pub fn create_page(state: Rc<RefCell<WelcomeState>>) -> gtk4::ScrolledWindow {
    let page = PreferencesPage::new();

    // Header
    let header_box = Box::new(Orientation::Vertical, 8);
    header_box.set_margin_top(24);
    header_box.set_margin_bottom(12);
    header_box.set_halign(gtk4::Align::Center);

    let title = Label::new(Some("Setup de Gaming"));
    title.add_css_class("title-1");
    header_box.append(&title);

    let icon = gtk4::Image::from_icon_name("input-gaming-symbolic");
    icon.set_pixel_size(64);
    icon.add_css_class("accent");
    header_box.append(&icon);

    let subtitle = Label::new(Some("Configure seu PC para jogos (opcional)"));
    subtitle.add_css_class("dim-label");
    header_box.append(&subtitle);

    let header_group = PreferencesGroup::new();
    header_group.add(&header_box);
    page.add(&header_group);

    // Gaming platforms
    let platforms_group = PreferencesGroup::builder()
        .title("Plataformas de Jogos")
        .description("Lojas e lancadores de jogos")
        .build();

    let platforms = [
        ("Steam", "A maior plataforma de jogos para PC", "steam", true),
        ("Lutris", "Gerenciador de jogos com suporte a Wine", "lutris", true),
        ("Heroic Games", "Launcher para Epic Games e GOG", "heroic-games-launcher", false),
        ("Bottles", "Execute aplicativos Windows facilmente", "bottles", false),
        ("itch.io", "Plataforma de jogos indie", "itch", false),
        ("Minigalaxy", "Cliente GOG nativo para Linux", "minigalaxy", false),
    ];

    for (name, desc, package, default) in platforms {
        let row = create_gaming_row(name, desc, package, default, "gaming_platforms", state.clone());
        platforms_group.add(&row);
    }

    page.add(&platforms_group);

    // Compatibility tools
    let compat_group = PreferencesGroup::builder()
        .title("Ferramentas de Compatibilidade")
        .description("Execute jogos Windows no Linux")
        .build();

    let compat_tools = [
        ("Wine", "Camada de compatibilidade Windows", "wine", true),
        ("Proton-GE", "Versao customizada do Proton da Valve", "proton-ge-custom", true),
        ("DXVK", "Traducao DirectX para Vulkan", "dxvk-bin", true),
        ("VKD3D", "Traducao DirectX 12 para Vulkan", "vkd3d-proton-bin", false),
        ("GameMode", "Otimizacoes de performance para jogos", "gamemode", true),
        ("MangoHud", "Overlay de performance para jogos", "mangohud", true),
    ];

    for (name, desc, package, default) in compat_tools {
        let row = create_gaming_row(name, desc, package, default, "gaming_tools", state.clone());
        compat_group.add(&row);
    }

    page.add(&compat_group);

    // Emulators
    let emulators_group = PreferencesGroup::builder()
        .title("Emuladores")
        .description("Jogue titulos retro e de outras plataformas")
        .build();

    let emulators = [
        ("RetroArch", "Frontend para emuladores de multiplas plataformas", "retroarch", false),
        ("PCSX2", "Emulador de PlayStation 2", "pcsx2", false),
        ("RPCS3", "Emulador de PlayStation 3", "rpcs3-bin", false),
        ("Dolphin", "Emulador de GameCube e Wii", "dolphin-emu", false),
        ("Yuzu", "Emulador de Nintendo Switch", "yuzu-mainline-bin", false),
        ("Cemu", "Emulador de Wii U", "cemu", false),
        ("PPSSPP", "Emulador de PSP", "ppsspp", false),
        ("Citra", "Emulador de Nintendo 3DS", "citra-qt-bin", false),
        ("DeSmuME", "Emulador de Nintendo DS", "desmume", false),
        ("mGBA", "Emulador de Game Boy Advance", "mgba-qt", false),
    ];

    for (name, desc, package, default) in emulators {
        let row = create_gaming_row(name, desc, package, default, "gaming_emulators", state.clone());
        emulators_group.add(&row);
    }

    page.add(&emulators_group);

    // Controller support
    let controller_group = PreferencesGroup::builder()
        .title("Suporte a Controles")
        .description("Drivers e ferramentas para gamepads")
        .build();

    let controller_tools = [
        ("Steam Input", "Suporte a controles via Steam", "steam", false),
        ("AntiMicroX", "Mapeamento de controle para teclado", "antimicrox", false),
        ("sc-controller", "Driver para Steam Controller", "sc-controller", false),
        ("xpadneo", "Driver Xbox One Bluetooth", "xpadneo-dkms", false),
        ("ds4drv", "Driver para DualShock 4", "ds4drv", false),
    ];

    for (name, desc, package, default) in controller_tools {
        let row = create_gaming_row(name, desc, package, default, "gaming_controllers", state.clone());
        controller_group.add(&row);
    }

    page.add(&controller_group);

    // Performance tips
    let tips_group = PreferencesGroup::builder()
        .title("Dicas de Performance")
        .build();

    let tips_text = Label::new(Some(
        "Para melhor performance em jogos:\n\
         - Use drivers proprietarios da NVIDIA ou AMDGPU\n\
         - Habilite o modo Gaming nas configuracoes de performance\n\
         - Configure o GameMode para otimizacoes automaticas\n\
         - Use MangoHud para monitorar FPS e uso de hardware"
    ));
    tips_text.add_css_class("dim-label");
    tips_text.add_css_class("caption");
    tips_text.set_wrap(true);
    tips_text.set_halign(gtk4::Align::Start);
    tips_text.set_margin_top(8);
    tips_text.set_margin_bottom(8);
    tips_text.set_margin_start(16);
    tips_group.add(&tips_text);

    page.add(&tips_group);

    let scrolled = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&page)
        .build();

    scrolled
}

fn create_gaming_row(
    name: &str,
    description: &str,
    package: &str,
    default: bool,
    category: &str,
    state: Rc<RefCell<WelcomeState>>,
) -> ActionRow {
    let row = ActionRow::builder()
        .title(name)
        .subtitle(description)
        .activatable(true)
        .build();

    let check = CheckButton::new();
    check.set_active(default);
    check.set_valign(gtk4::Align::Center);
    row.add_suffix(&check);
    row.set_activatable_widget(Some(&check));

    let package_name = package.to_string();
    let category_name = category.to_string();
    let state_clone = state.clone();

    // Initialize default
    if default {
        let mut state = state.borrow_mut();
        match category {
            "gaming_platforms" => {
                if !state.gaming_platforms.contains(&package_name) {
                    state.gaming_platforms.push(package_name.clone());
                }
            }
            "gaming_emulators" => {
                if !state.gaming_emulators.contains(&package_name) {
                    state.gaming_emulators.push(package_name.clone());
                }
            }
            _ => {}
        }
    }

    check.connect_toggled(move |btn| {
        let mut state = state_clone.borrow_mut();
        let list = match category_name.as_str() {
            "gaming_platforms" => &mut state.gaming_platforms,
            "gaming_emulators" => &mut state.gaming_emulators,
            _ => return,
        };

        if btn.is_active() {
            if !list.contains(&package_name) {
                list.push(package_name.clone());
            }
        } else {
            list.retain(|x| x != &package_name);
        }
    });

    row
}
