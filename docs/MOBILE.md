# Winux OS - Desenvolvimento Mobile

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║    ███╗   ███╗ ██████╗ ██████╗ ██╗██╗     ███████╗    ██████╗ ███████╗██╗   ██║
║    ████╗ ████║██╔═══██╗██╔══██╗██║██║     ██╔════╝    ██╔══██╗██╔════╝██║   ██║
║    ██╔████╔██║██║   ██║██████╔╝██║██║     █████╗      ██║  ██║█████╗  ██║   ██║
║    ██║╚██╔╝██║██║   ██║██╔══██╗██║██║     ██╔══╝      ██║  ██║██╔══╝  ╚██╗ ██╔╝
║    ██║ ╚═╝ ██║╚██████╔╝██████╔╝██║███████╗███████╗    ██████╔╝███████╗ ╚████╔╝ ║
║    ╚═╝     ╚═╝ ╚═════╝ ╚═════╝ ╚═╝╚══════╝╚══════╝    ╚═════╝ ╚══════╝  ╚═══╝  ║
║                                                                               ║
║              Android | iOS | Flutter | React Native | Swift                   ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

Este guia cobre o desenvolvimento de aplicativos mobile no Winux OS, incluindo Android nativo, iOS (com limitacoes), frameworks cross-platform e Swift no Linux.

---

## Indice

1. [Visao Geral](#visao-geral)
2. [Desenvolvimento Android](#desenvolvimento-android)
3. [Desenvolvimento iOS](#desenvolvimento-ios)
4. [Flutter](#flutter)
5. [React Native](#react-native)
6. [Swift no Linux](#swift-no-linux)
7. [Ferramentas Uteis](#ferramentas-uteis)
8. [Troubleshooting](#troubleshooting)

---

## Visao Geral

### Plataformas Suportadas

```
┌─────────────────────────────────────────────────────────────────┐
│                    MOBILE DEV NO WINUX                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │   ANDROID   │    │     iOS     │    │   CROSS     │        │
│   │  ━━━━━━━━━  │    │  ━━━━━━━━━  │    │  ━━━━━━━━━  │        │
│   │ ✓ Native    │    │ ⚠ Limited   │    │ ✓ Flutter   │        │
│   │ ✓ Kotlin    │    │ ⚠ Swift*    │    │ ✓ React     │        │
│   │ ✓ Java      │    │ ✗ Xcode     │    │   Native    │        │
│   │ ✓ NDK       │    │             │    │ ✓ Capacitor │        │
│   │ ✓ Emulator  │    │             │    │             │        │
│   └─────────────┘    └─────────────┘    └─────────────┘        │
│                                                                 │
│   * Swift compila no Linux mas sem UIKit/SwiftUI               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Capacidades por Plataforma

| Feature | Android | iOS | Cross-Platform |
|:--------|:-------:|:---:|:--------------:|
| Desenvolvimento nativo | ✅ | ⚠️ | N/A |
| Emulador/Simulador | ✅ | ❌ | ✅ (Android) |
| Build final | ✅ | ❌* | ✅ (Android) |
| Hot reload | ✅ | ✅** | ✅ |
| Debug | ✅ | ⚠️ | ✅ |
| Publicacao | ✅ | ❌* | ✅ (Android) |

\* Requer macOS para build final e publicacao iOS
\** Apenas em dispositivo fisico via rede

---

## Desenvolvimento Android

### Setup do Ambiente

O Winux vem com Android SDK pre-configurado. Para verificar:

```bash
# Verificar instalacao
echo $ANDROID_HOME
# /home/user/Android/Sdk

# Verificar SDK Manager
sdkmanager --list

# Verificar ADB
adb --version
```

### Instalacao Manual (se necessario)

```bash
# Baixar Command Line Tools
wget https://dl.google.com/android/repository/commandlinetools-linux-latest.zip

# Extrair
unzip commandlinetools-linux-latest.zip -d ~/Android/Sdk/cmdline-tools/
mv ~/Android/Sdk/cmdline-tools/cmdline-tools ~/Android/Sdk/cmdline-tools/latest

# Configurar PATH
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
export PATH="$ANDROID_HOME/platform-tools:$PATH"
export PATH="$ANDROID_HOME/emulator:$PATH"

# Aceitar licencas
yes | sdkmanager --licenses

# Instalar componentes essenciais
sdkmanager "platform-tools" "platforms;android-34" "build-tools;34.0.0"
sdkmanager "system-images;android-34;google_apis;x86_64"
sdkmanager "emulator"
```

### Criar AVD (Android Virtual Device)

```bash
# Listar system images disponiveis
sdkmanager --list | grep system-images

# Criar AVD
avdmanager create avd \
    --name "Pixel_6_API_34" \
    --package "system-images;android-34;google_apis;x86_64" \
    --device "pixel_6"

# Listar AVDs
avdmanager list avd

# Iniciar emulador
emulator -avd Pixel_6_API_34
```

### Aceleracao de Hardware (KVM)

```bash
# Verificar suporte KVM
kvm-ok

# Se nao estiver habilitado
sudo apt install qemu-kvm
sudo adduser $USER kvm

# Reiniciar sessao e testar
emulator -avd Pixel_6_API_34 -accel on
```

### Projeto Android Nativo (Kotlin)

```bash
# Usando Android Studio (recomendado)
# Baixe de: https://developer.android.com/studio

# Ou via linha de comando com Gradle
mkdir MeuApp && cd MeuApp

# Criar projeto manualmente ou usar template
gradle init --type kotlin-application

# build.gradle.kts
plugins {
    id("com.android.application") version "8.2.0"
    id("org.jetbrains.kotlin.android") version "1.9.22"
}

android {
    namespace = "com.exemplo.meuapp"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.exemplo.meuapp"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"
    }
}
```

### Build e Deploy

```bash
# Build debug
./gradlew assembleDebug

# Build release
./gradlew assembleRelease

# Instalar no dispositivo/emulador
adb install app/build/outputs/apk/debug/app-debug.apk

# Logcat
adb logcat | grep "MeuApp"

# Debug wireless
adb tcpip 5555
adb connect 192.168.1.100:5555
```

### NDK (Native Development Kit)

```bash
# Instalar NDK
sdkmanager "ndk;26.1.10909125"

# Configurar
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"

# CMakeLists.txt para JNI
cmake_minimum_required(VERSION 3.18.1)
project(native-lib)

add_library(native-lib SHARED native-lib.cpp)

find_library(log-lib log)
target_link_libraries(native-lib ${log-lib})
```

---

## Desenvolvimento iOS

### Limitacoes no Linux

> **Importante**: O desenvolvimento iOS completo requer macOS com Xcode. No Linux, temos algumas alternativas limitadas.

```
┌─────────────────────────────────────────────────────────────────┐
│                 iOS NO LINUX - LIMITACOES                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ❌ Nao disponivel:                                             │
│     - Xcode                                                     │
│     - iOS Simulator                                             │
│     - Build de .ipa assinado                                    │
│     - Publicacao na App Store                                   │
│     - Interface Builder / SwiftUI previews                      │
│                                                                 │
│  ✅ Disponivel:                                                 │
│     - Swift compiler (sem UIKit)                                │
│     - Desenvolvimento de logica de negocios                     │
│     - Testes unitarios de codigo Swift                          │
│     - Cross-platform via Flutter/React Native                   │
│     - Deploy para dispositivo via rede (experimental)           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Workarounds

#### 1. Desenvolvimento Remoto (macOS via SSH/VNC)

```bash
# Conectar a um Mac remoto
ssh usuario@mac.local

# Usar Xcode via linha de comando
xcodebuild -project MeuApp.xcodeproj -scheme MeuApp build

# Ou usar VNC para interface grafica
vinagre mac.local:5900
```

#### 2. GitHub Actions / CI para Build iOS

```yaml
# .github/workflows/ios.yml
name: iOS Build

on: [push]

jobs:
  build:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: |
          xcodebuild -project MeuApp.xcodeproj \
            -scheme MeuApp \
            -sdk iphonesimulator \
            -destination 'platform=iOS Simulator,name=iPhone 15' \
            build

      - name: Upload IPA
        uses: actions/upload-artifact@v4
        with:
          name: app-ipa
          path: build/MeuApp.ipa
```

#### 3. Servicos de Build na Nuvem

- **Codemagic**: CI/CD para mobile com macOS
- **Bitrise**: Build iOS/Android na nuvem
- **App Center**: Microsoft mobile DevOps

#### 4. Theos (Jailbreak Development)

```bash
# Para desenvolvimento de tweaks iOS (requer dispositivo jailbroken)
git clone --recursive https://github.com/theos/theos.git ~/theos
export THEOS=~/theos

# Criar projeto
$THEOS/bin/nic.pl
```

---

## Flutter

### Instalacao

```bash
# Baixar Flutter SDK
git clone https://github.com/flutter/flutter.git ~/flutter -b stable

# Adicionar ao PATH
export PATH="$HOME/flutter/bin:$PATH"

# Verificar instalacao
flutter doctor

# Aceitar licencas Android
flutter doctor --android-licenses
```

### Flutter Doctor Output Esperado

```
Doctor summary (to see all details, run flutter doctor -v):
[✓] Flutter (Channel stable, 3.19.x)
[✓] Android toolchain - develop for Android devices
[✓] Linux toolchain - develop for Linux desktop
[✓] Chrome - develop for the web
[!] Android Studio (not installed)
[✓] VS Code
[✓] Connected device (2 available)
[✓] Network resources
```

### Criar Projeto Flutter

```bash
# Criar novo projeto
flutter create meu_app
cd meu_app

# Rodar no emulador
flutter emulators --launch Pixel_6_API_34
flutter run

# Hot reload: pressione 'r' no terminal
# Hot restart: pressione 'R' no terminal
```

### Estrutura do Projeto

```
meu_app/
├── android/          # Projeto Android nativo
├── ios/              # Projeto iOS nativo
├── lib/              # Codigo Dart
│   └── main.dart
├── test/             # Testes
├── web/              # Web app
├── linux/            # Linux desktop
├── windows/          # Windows desktop
├── macos/            # macOS desktop
└── pubspec.yaml      # Dependencias
```

### Exemplo Basico

```dart
// lib/main.dart
import 'package:flutter/material.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Meu App',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Home'),
      ),
      body: const Center(
        child: Text('Bem-vindo ao Flutter!'),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {},
        child: const Icon(Icons.add),
      ),
    );
  }
}
```

### Build para Producao

```bash
# Android APK
flutter build apk --release

# Android App Bundle (para Play Store)
flutter build appbundle --release

# Web
flutter build web --release

# Linux
flutter build linux --release

# iOS (requer macOS)
# flutter build ios --release
```

### Plugins Uteis

```yaml
# pubspec.yaml
dependencies:
  flutter:
    sdk: flutter

  # HTTP requests
  http: ^1.2.0
  dio: ^5.4.0

  # State management
  provider: ^6.1.1
  riverpod: ^2.5.0
  bloc: ^8.1.3

  # Storage
  shared_preferences: ^2.2.2
  sqflite: ^2.3.2
  hive: ^2.2.3

  # Firebase
  firebase_core: ^2.25.4
  firebase_auth: ^4.17.4
  cloud_firestore: ^4.15.4

  # UI
  flutter_svg: ^2.0.9
  cached_network_image: ^3.3.1
```

---

## React Native

### Instalacao

```bash
# Verificar Node.js
node --version  # 18+

# Instalar React Native CLI
npm install -g react-native-cli

# Ou usar npx (recomendado)
npx react-native --version
```

### Criar Projeto

```bash
# Criar projeto com template TypeScript
npx react-native init MeuApp --template react-native-template-typescript

cd MeuApp

# Instalar dependencias
npm install

# Iniciar Metro bundler
npm start

# Em outro terminal, rodar no Android
npm run android
```

### Estrutura do Projeto

```
MeuApp/
├── android/          # Projeto Android nativo
├── ios/              # Projeto iOS nativo
├── src/              # Codigo fonte (criar)
│   ├── components/
│   ├── screens/
│   ├── navigation/
│   └── services/
├── App.tsx           # Componente raiz
├── index.js          # Entry point
├── package.json
├── tsconfig.json
└── metro.config.js
```

### Exemplo Basico

```tsx
// App.tsx
import React from 'react';
import {
  SafeAreaView,
  StyleSheet,
  Text,
  View,
  TouchableOpacity,
} from 'react-native';

function App(): React.JSX.Element {
  return (
    <SafeAreaView style={styles.container}>
      <View style={styles.content}>
        <Text style={styles.title}>Bem-vindo!</Text>
        <Text style={styles.subtitle}>
          Meu primeiro app React Native
        </Text>
        <TouchableOpacity style={styles.button}>
          <Text style={styles.buttonText}>Comecar</Text>
        </TouchableOpacity>
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
  },
  content: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    marginBottom: 10,
  },
  subtitle: {
    fontSize: 16,
    color: '#666',
    marginBottom: 30,
  },
  button: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 30,
    paddingVertical: 15,
    borderRadius: 10,
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});

export default App;
```

### Navegacao

```bash
# Instalar React Navigation
npm install @react-navigation/native
npm install @react-navigation/native-stack
npm install react-native-screens react-native-safe-area-context
```

```tsx
// navigation/AppNavigator.tsx
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import HomeScreen from '../screens/HomeScreen';
import DetailsScreen from '../screens/DetailsScreen';

const Stack = createNativeStackNavigator();

export default function AppNavigator() {
  return (
    <NavigationContainer>
      <Stack.Navigator>
        <Stack.Screen name="Home" component={HomeScreen} />
        <Stack.Screen name="Details" component={DetailsScreen} />
      </Stack.Navigator>
    </NavigationContainer>
  );
}
```

### Build para Producao

```bash
# Android APK
cd android
./gradlew assembleRelease

# Android App Bundle
./gradlew bundleRelease

# APK em: android/app/build/outputs/apk/release/
```

---

## Swift no Linux

### Instalacao

```bash
# Baixar Swift
wget https://download.swift.org/swift-5.9.2-release/ubuntu2204/swift-5.9.2-RELEASE/swift-5.9.2-RELEASE-ubuntu22.04.tar.gz

# Extrair
tar xzf swift-5.9.2-RELEASE-ubuntu22.04.tar.gz
sudo mv swift-5.9.2-RELEASE-ubuntu22.04 /usr/local/swift

# Adicionar ao PATH
export PATH="/usr/local/swift/usr/bin:$PATH"

# Verificar
swift --version
```

### Limitacoes no Linux

```
┌─────────────────────────────────────────────────────────────────┐
│                 SWIFT NO LINUX                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ✅ Disponivel:                                                 │
│     - Swift compiler                                            │
│     - Swift Package Manager (SPM)                               │
│     - Foundation (parcial)                                      │
│     - Vapor (web framework)                                     │
│     - Perfect, Kitura (web frameworks)                          │
│     - SwiftNIO (networking)                                     │
│     - Codable, Combine                                          │
│                                                                 │
│  ❌ Nao disponivel:                                             │
│     - UIKit                                                     │
│     - SwiftUI                                                   │
│     - AppKit                                                    │
│     - CoreData                                                  │
│     - CoreGraphics                                              │
│     - Qualquer framework Apple-only                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Uso Principal: Backend/Server-Side

```bash
# Criar projeto Swift
mkdir MeuServidor && cd MeuServidor
swift package init --type executable

# Package.swift
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MeuServidor",
    platforms: [.macOS(.v12)],
    dependencies: [
        .package(url: "https://github.com/vapor/vapor.git", from: "4.0.0"),
    ],
    targets: [
        .executableTarget(
            name: "MeuServidor",
            dependencies: [
                .product(name: "Vapor", package: "vapor"),
            ]
        ),
    ]
)
```

### Exemplo com Vapor

```swift
// Sources/MeuServidor/main.swift
import Vapor

let app = try Application(.detect())
defer { app.shutdown() }

app.get { req in
    return "Ola, Winux!"
}

app.get("hello", ":name") { req -> String in
    let name = req.parameters.get("name")!
    return "Ola, \(name)!"
}

try app.run()
```

```bash
# Build e executar
swift build
swift run

# Testar
curl http://localhost:8080/hello/World
```

### Compartilhando Codigo com iOS

Uma estrategia comum e criar packages Swift com logica de negocios que funcionam em ambas as plataformas:

```
MeuProjeto/
├── MeuAppCore/           # Package Swift (funciona no Linux)
│   ├── Package.swift
│   └── Sources/
│       └── MeuAppCore/
│           ├── Models/
│           ├── Services/
│           └── Utils/
│
├── MeuAppIOS/            # Projeto iOS (requer macOS)
│   └── usa MeuAppCore como dependencia
│
└── MeuAppServer/         # Servidor Vapor (funciona no Linux)
    └── usa MeuAppCore como dependencia
```

---

## Ferramentas Uteis

### Scrcpy - Espelhamento Android

```bash
# Instalar
sudo apt install scrcpy

# Usar (com dispositivo conectado)
scrcpy

# Com opcoes
scrcpy --max-size 1024 --bit-rate 2M
scrcpy --record video.mp4
```

### Android Debug Bridge (ADB)

```bash
# Listar dispositivos
adb devices

# Instalar APK
adb install app.apk

# Logcat filtrado
adb logcat -s "MeuApp"

# Screenshot
adb exec-out screencap -p > screen.png

# Gravar tela
adb shell screenrecord /sdcard/video.mp4
adb pull /sdcard/video.mp4

# Shell do dispositivo
adb shell

# File transfer
adb push local.txt /sdcard/
adb pull /sdcard/remote.txt
```

### Firebase CLI

```bash
# Instalar
npm install -g firebase-tools

# Login
firebase login

# Inicializar projeto
firebase init

# Deploy
firebase deploy
```

---

## Troubleshooting

### Emulador Android Lento

```bash
# Verificar KVM
sudo apt install cpu-checker
kvm-ok

# Habilitar aceleracao
emulator -avd MeuAVD -accel on -gpu host
```

### Flutter: "Android SDK not found"

```bash
# Configurar manualmente
flutter config --android-sdk $ANDROID_HOME
flutter doctor
```

### React Native: "SDK location not found"

```bash
# Criar local.properties
echo "sdk.dir=$ANDROID_HOME" > android/local.properties
```

### ADB: "device unauthorized"

```bash
# No dispositivo: habilitar debug USB e autorizar PC
adb kill-server
adb start-server
adb devices
```

### Gradle: Out of Memory

```bash
# Aumentar memoria
echo "org.gradle.jvmargs=-Xmx4096m" >> ~/.gradle/gradle.properties
```

---

## Resumo

| Plataforma | Suporte | Recomendacao |
|:-----------|:--------|:-------------|
| **Android** | Completo | Desenvolvimento nativo ou Flutter |
| **iOS** | Parcial | Flutter + CI/CD no macOS |
| **Cross** | Completo | Flutter (melhor performance) ou React Native |
| **Backend** | Completo | Swift com Vapor ou Node.js |

---

**Winux OS Project - 2026**

*Desenvolvimento mobile no Linux e possivel!*
