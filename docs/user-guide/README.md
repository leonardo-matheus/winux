# Winux OS - Guia do Usuario

> **Versao 1.0 Aurora**
> O Melhor dos Dois Mundos: Gaming + Produtividade

```
██╗    ██╗██╗███╗   ██╗██╗   ██╗██╗  ██╗
██║    ██║██║████╗  ██║██║   ██║╚██╗██╔╝
██║ █╗ ██║██║██╔██╗ ██║██║   ██║ ╚███╔╝
██║███╗██║██║██║╚██╗██║██║   ██║ ██╔██╗
╚███╔███╔╝██║██║ ╚████║╚██████╔╝██╔╝ ██╗
 ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝
```

---

## Indice

1. [Instalacao](#instalacao)
2. [Primeiro Uso](#primeiro-uso)
3. [Desktop e Aplicativos](#desktop-e-aplicativos)
4. [Gaming](#gaming)
5. [Personalizacao](#personalizacao)
6. [FAQ](#faq)

---

## Instalacao

### Requisitos do Sistema

#### Minimos
| Componente | Requisito |
|------------|-----------|
| Processador | x86_64 com suporte SSE4.2 |
| Memoria RAM | 4 GB |
| Armazenamento | 30 GB (SSD recomendado) |
| Placa de Video | Vulkan 1.1 compativel |
| Conexao | Internet para download de drivers |

#### Recomendados
| Componente | Requisito |
|------------|-----------|
| Processador | AMD Ryzen 5 / Intel Core i5 (6+ cores) |
| Memoria RAM | 16 GB DDR4/DDR5 |
| Armazenamento | 100 GB NVMe SSD |
| Placa de Video | NVIDIA RTX 3060 / AMD RX 6700 XT ou superior |

### Download da ISO

1. Acesse a pagina oficial: `https://winux-os.org/download`
2. Escolha a versao:
   - **Winux OS 1.0 Aurora** - Versao estavel recomendada
   - **Winux OS Rolling** - Versao com ultimas atualizacoes
3. Verifique a integridade do download:
   ```bash
   sha256sum -c winux-1.0-aurora-amd64.iso.sha256
   ```

### Criando Midia de Instalacao

#### No Linux
```bash
# Substitua /dev/sdX pelo seu pendrive
sudo dd if=winux-1.0-aurora-amd64.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

#### No Windows
1. Baixe o [Rufus](https://rufus.ie) ou [balenaEtcher](https://www.balena.io/etcher/)
2. Selecione a ISO do Winux OS
3. Selecione seu pendrive (minimo 8 GB)
4. Clique em "Iniciar" e aguarde a gravacao

### Processo de Instalacao

1. **Boot pelo Pendrive**
   - Reinicie o computador e acesse o menu de boot (F12, F2, Del ou Esc)
   - Selecione o pendrive com Winux OS

2. **Tela de Boas-Vindas**
   - Escolha "Winux OS - Live Session" para testar
   - Escolha "Winux OS - Install" para instalacao direta

3. **Configuracao Inicial**
   - Idioma: Portugues (Brasil)
   - Layout do Teclado: ABNT2 ou US International
   - Fuso Horario: America/Sao_Paulo

4. **Particionamento**
   - **Automatico**: Recomendado para iniciantes
   - **Manual**: Para usuarios avancados

   Esquema recomendado:
   ```
   /boot/efi  - 512 MB (FAT32, para UEFI)
   /          - 50 GB  (ext4 ou btrfs)
   /home      - Restante (ext4 ou btrfs)
   swap       - 8 GB   (ou arquivo de swap)
   ```

5. **Configuracao de Usuario**
   - Nome completo
   - Nome de usuario (sem espacos)
   - Senha (minimo 8 caracteres)
   - Nome do computador

6. **Instalacao**
   - Aguarde a copia dos arquivos (15-30 minutos)
   - Reinicie quando solicitado
   - Remova o pendrive

---

## Primeiro Uso

### Tela de Login

Apos a instalacao, voce vera a tela de login do Winux OS. Digite sua senha para acessar o desktop.

### Assistente de Configuracao Inicial

Na primeira inicializacao, o assistente guiara voce:

1. **Conexao de Rede**
   - Wi-Fi: Selecione sua rede e digite a senha
   - Ethernet: Configurado automaticamente

2. **Atualizacoes do Sistema**
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

3. **Drivers de Video**
   - NVIDIA: Detectado automaticamente
   - AMD: Drivers Mesa instalados por padrao
   - Intel: Suporte nativo incluido

4. **Configuracao de Gaming**
   - Steam sera configurado automaticamente
   - Wine/Proton otimizado para jogos

### Atalhos Essenciais

| Atalho | Acao |
|--------|------|
| `Super` | Abrir Menu Iniciar |
| `Super + E` | Abrir Gerenciador de Arquivos |
| `Super + T` | Abrir Terminal |
| `Super + D` | Mostrar Desktop |
| `Super + L` | Bloquear Tela |
| `Alt + Tab` | Alternar Janelas |
| `Alt + F4` | Fechar Janela Atual |
| `Ctrl + Alt + Del` | Menu de Sistema |
| `Print Screen` | Captura de Tela |
| `Super + Shift + S` | Captura de Area |

---

## Desktop e Aplicativos

### Interface do Desktop

O Winux OS utiliza uma interface inspirada no Windows 11 com Fluent Design:

```
+----------------------------------------------------------+
|  [Apps] [Pesquisa]          [Tray] [Relogio] [Notif]    |
+----------------------------------------------------------+
|                                                          |
|                                                          |
|                      DESKTOP                             |
|                                                          |
|                                                          |
+----------------------------------------------------------+
| [Iniciar] | [Apps Fixados] |          [Janelas Abertas] |
+----------------------------------------------------------+
```

### Aplicativos Nativos Winux

#### Winux Files
Gerenciador de arquivos moderno e rapido.

**Recursos:**
- Navegacao em abas
- Visualizacao em grade e lista
- Preview de arquivos
- Compactacao/Descompactacao integrada
- Acesso rapido a locais favoritos

**Atalhos:**
- `Ctrl + N` - Nova janela
- `Ctrl + T` - Nova aba
- `Ctrl + W` - Fechar aba
- `F2` - Renomear
- `Delete` - Mover para lixeira
- `Shift + Delete` - Excluir permanentemente

#### Winux Terminal
Terminal moderno com suporte a abas e temas.

**Recursos:**
- Multiplas abas e paineis
- Temas customizaveis
- Suporte a perfis
- Transparencia e acrilico
- Integracao com shells (bash, zsh, fish)

**Atalhos:**
- `Ctrl + Shift + T` - Nova aba
- `Ctrl + Shift + N` - Nova janela
- `Ctrl + +/-` - Zoom
- `Ctrl + Shift + C/V` - Copiar/Colar

#### Winux Settings
Central de configuracoes do sistema.

**Categorias:**
- Sistema (informacoes, atualizacoes)
- Rede (Wi-Fi, Ethernet, VPN)
- Personalizacao (temas, papel de parede)
- Aplicativos (padrao, inicializacao)
- Contas (usuarios, sincronizacao)
- Privacidade (permissoes, historico)
- Acessibilidade (lupa, narrador)
- Gaming (performance, compatibilidade)

#### Winux Store
Loja de aplicativos integrada.

**Fontes de Software:**
- Repositorios APT nativos
- Flatpak (Flathub)
- Snap Store
- AppImage

**Categorias:**
- Produtividade
- Jogos
- Multimidia
- Desenvolvimento
- Utilitarios

#### Winux Monitor
Monitor de sistema e processos.

**Abas:**
- Processos (CPU, memoria, disco por processo)
- Desempenho (graficos em tempo real)
- Historico de aplicativos
- Inicializacao
- Usuarios
- Detalhes
- Servicos

#### Winux Edit
Editor de texto simples e rapido.

**Recursos:**
- Destaque de sintaxe
- Numeracao de linhas
- Localizacao e substituicao
- Suporte a encodings
- Modo escuro

---

## Gaming

### Visao Geral

O Winux OS foi projetado para gaming, com otimizacoes de kernel, drivers e compatibilidade com jogos Windows.

### Wine - Executando Programas Windows

Wine permite executar programas Windows nativamente no Linux.

#### Configuracao Basica
```bash
# O Wine ja vem configurado. Para verificar:
wine --version

# Executar um programa Windows:
wine programa.exe

# Usar o launcher otimizado do Winux:
winux-run programa.exe
```

#### Gerenciador de Prefixos
```bash
# Criar novo prefix (ambiente Windows isolado)
WINEPREFIX=~/.wine-jogos winecfg

# Configurar versao do Windows
winecfg  # Selecione Windows 10 na aba "Aplicativos"
```

#### Instalando Dependencias
```bash
# Visual C++ Runtimes
winetricks vcrun2019 vcrun2022

# DirectX
winetricks d3dx9 d3dx11_43 d3dcompiler_47

# .NET Framework
winetricks dotnet48

# Fontes
winetricks corefonts
```

### Proton - Gaming com Steam

Proton e a camada de compatibilidade da Valve baseada no Wine.

#### Configuracao no Steam
1. Abra o Steam
2. Acesse: Steam > Configuracoes > Compatibilidade
3. Marque "Ativar Steam Play para todos os titulos"
4. Selecione a versao do Proton (recomendado: Proton Experimental)

#### Proton-GE (Recomendado)
Versao otimizada com patches adicionais:

```bash
# Instalar Proton-GE
/usr/share/winux/scripts/proton-install.sh

# Ou manualmente via ProtonUp-Qt
flatpak install flathub net.davidotek.pupgui2
```

### DXVK e VKD3D

Traducao de DirectX para Vulkan para melhor performance.

#### DXVK (DirectX 9/10/11)
```bash
# Ja vem instalado. Para atualizar:
winetricks dxvk

# Verificar se esta ativo (em jogo):
# Pressione F10 para ver HUD com info do DXVK
```

#### VKD3D-Proton (DirectX 12)
```bash
# Instalacao:
winetricks vkd3d

# Para jogos especificos:
VKD3D_CONFIG=dxr wine jogo_dx12.exe
```

### Steam no Winux OS

#### Instalacao
```bash
# Steam ja vem pre-instalado
# Se necessario reinstalar:
sudo apt install steam
```

#### Otimizacoes
```bash
# Habilitar shader pre-caching
# Steam > Configuracoes > Shader Pre-Caching > Ativar

# Usar Proton para jogos Windows
# Biblioteca > Jogo > Propriedades > Compatibilidade > Proton
```

#### Integracao com Gamepad
- Xbox Controllers: Suporte nativo via xpad
- PlayStation Controllers: Suporte via hid-playstation
- Controles genericos: Configurar no Steam Big Picture

### GameMode

Otimiza automaticamente o sistema durante jogos.

```bash
# Verificar se esta ativo
gamemoded -s

# Executar jogo com GameMode
gamemoderun wine jogo.exe

# No Steam, adicionar ao comando de lancamento:
gamemoderun %command%
```

### MangoHud

Overlay para monitorar performance em jogos.

```bash
# Ativar MangoHud
mangohud wine jogo.exe

# No Steam:
mangohud %command%

# Configurar (criar ~/.config/MangoHud/MangoHud.conf):
fps
cpu_temp
gpu_temp
ram
vram
frame_timing
```

### Dicas de Performance

1. **Use o perfil de energia "Performance"**
   ```bash
   powerprofilesctl set performance
   ```

2. **Desative compositing durante jogos**
   - Automatico no Winux Shell para jogos em fullscreen

3. **Configure o limite de FPS**
   ```bash
   # No MangoHud.conf:
   fps_limit=144
   ```

4. **Monitore temperaturas**
   ```bash
   # CPU
   sensors

   # GPU NVIDIA
   nvidia-smi

   # GPU AMD
   cat /sys/class/drm/card0/device/hwmon/hwmon*/temp1_input
   ```

---

## Personalizacao

### Temas

O Winux OS inclui temas Fluent Design:

#### Modo Claro/Escuro
```
Configuracoes > Personalizacao > Cores > Modo de cor
```

#### Temas Incluidos
- **Winux Light** - Tema claro padrao
- **Winux Dark** - Tema escuro
- **Winux Midnight** - Tema OLED puro
- **Winux Aurora** - Tema com acentos coloridos

### Papel de Parede

```
Configuracoes > Personalizacao > Plano de Fundo
```

Opcoes:
- Imagem estatica
- Slideshow
- Cor solida
- Wallpaper Engine (via Steam)

### Icones

Packs de icones incluidos:
- **Winux Icons** (padrao)
- **Papirus**
- **Fluent Icons**

```bash
# Instalar pack adicional
sudo apt install papirus-icon-theme
```

### Fontes

Fontes pre-instaladas otimizadas:
- **Segoe UI** - Interface do sistema
- **Cascadia Code** - Terminal e codigo
- **Noto Sans** - Suporte multilinguagem

```bash
# Instalar fontes Microsoft
sudo apt install ttf-mscorefonts-installer
```

### Efeitos Visuais

```
Configuracoes > Personalizacao > Efeitos
```

- Transparencia de janelas
- Efeito de acrilico/blur
- Animacoes de abertura
- Sombras de janela

### Painel e Taskbar

Personalize a barra de tarefas:
- Posicao (inferior, superior, lateral)
- Tamanho (pequeno, medio, grande)
- Comportamento (sempre visivel, auto-ocultar)
- Icones da bandeja do sistema

---

## FAQ

### Geral

**P: O Winux OS e gratuito?**
R: Sim, o Winux OS e totalmente gratuito e open-source sob licencas GPL v3 e MIT.

**P: Posso usar junto com Windows (dual boot)?**
R: Sim! Durante a instalacao, escolha particionar manualmente e mantenha a particao do Windows.

**P: Quanto espaco o Winux OS ocupa?**
R: A instalacao base ocupa aproximadamente 15 GB. Recomendamos 50 GB para uso confortavel.

### Hardware

**P: Minha placa de video sera suportada?**
R: O Winux OS suporta:
- NVIDIA: Drivers proprietarios incluidos
- AMD: Drivers Mesa open-source (excelente suporte)
- Intel: Suporte nativo completo

**P: Preciso desativar Secure Boot?**
R: Nao necessariamente. O Winux OS suporta Secure Boot, mas alguns drivers NVIDIA podem exigir desativacao.

**P: Meu hardware e muito antigo, funcionara?**
R: Se o hardware suporta SSE4.2 e Vulkan 1.1, deve funcionar. Para hardware mais antigo, considere distribuicoes leves.

### Gaming

**P: Todos os jogos Windows funcionam?**
R: A maioria funciona. Consulte [ProtonDB](https://www.protondb.com) para verificar compatibilidade especifica.

**P: Como saber se um jogo funciona?**
R:
1. Verifique no ProtonDB (protondb.com)
2. Procure no WineHQ AppDB (appdb.winehq.org)
3. Teste no Live USB antes de instalar

**P: O anti-cheat funciona?**
R: Depende do jogo:
- Easy Anti-Cheat: Muitos jogos suportam (Fortnite, Apex Legends, etc.)
- BattlEye: Suporte crescente
- Alguns anti-cheats ainda bloqueiam Linux

**P: A performance e pior que no Windows?**
R: Na maioria dos casos:
- Jogos nativos: Performance igual ou superior
- Jogos via Proton/Wine: 0-10% menor em media
- Alguns jogos Vulkan: Performance igual ou superior

### Software

**P: Posso instalar programas .exe?**
R: Sim, use o Wine:
```bash
wine programa.exe
# ou
winux-run programa.exe
```

**P: Como instalar o Microsoft Office?**
R: Opcoes:
1. Microsoft 365 Online (via navegador)
2. LibreOffice (alternativa gratuita compativel)
3. Wine (Office 2016 e anteriores)
4. CrossOver (solucao comercial)

**P: O Photoshop funciona?**
R: Versoes antigas funcionam via Wine. Alternativas:
- GIMP
- Krita
- Photopea (online)

### Problemas Comuns

**P: Tela preta apos instalacao**
R: Reinicie em modo de recuperacao e instale drivers:
```bash
sudo ubuntu-drivers autoinstall
```

**P: Wi-Fi nao funciona**
R: Verifique se o driver esta carregado:
```bash
lspci | grep -i wireless
sudo dmesg | grep -i wifi
```

**P: Som nao funciona**
R: Reinicie o PipeWire:
```bash
systemctl --user restart pipewire pipewire-pulse wireplumber
```

**P: Jogo trava ou nao inicia**
R: Tente:
1. Verificar logs: `PROTON_LOG=1 %command%`
2. Usar versao diferente do Proton
3. Instalar dependencias: `protontricks <appid> vcrun2019 dxvk`

### Suporte

**P: Onde buscar ajuda?**
R:
- Forum: forum.winux-os.org
- Discord: discord.gg/winux
- GitHub Issues: github.com/winux-os/winux/issues
- Wiki: wiki.winux-os.org

**P: Como reportar bugs?**
R: Use o GitHub Issues com:
1. Descricao do problema
2. Passos para reproduzir
3. Logs relevantes (`journalctl -b`)
4. Informacoes de hardware (`inxi -Fxz`)

---

## Proximos Passos

Apos configurar seu sistema:

1. **Explore a Winux Store** para descobrir novos aplicativos
2. **Configure o Steam** e baixe seus jogos
3. **Personalize** o sistema ao seu gosto
4. **Junte-se a comunidade** no Discord

---

**Winux OS Project - 2026**
*O Melhor dos Dois Mundos*
