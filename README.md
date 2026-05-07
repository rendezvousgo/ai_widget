# Quota — AI 사용량 위젯

Tauri 2 + Svelte 5 데스크탑 위젯. Claude Pro/Max 플랜 사용량(5h, Weekly 등)과 OpenAI RPM/TPM을 한 화면에 표시.

690×422 고정창, 라이트/다크 토글, 한국어/영어 토글, 자동시작, always-on-top.

---

## 사용자 — 설치만 할 거면

[Releases](https://github.com/rendezvousgo/ai_widget/releases)에서 OS에 맞는 파일 다운로드:

**정식 인스톨러 (시작 메뉴/제어판 등록):**
- **Windows**: `.msi` 또는 `.exe (NSIS)` — 더블클릭 설치 (코드 서명 안 됨 → SmartScreen "추가 정보 → 실행")
- **Linux**: `.AppImage` (실행 권한 주고 실행) 또는 `.deb` (Debian/Ubuntu)

**Portable (설치 없이 즉시 실행):**
- **Windows**: `Quota-vX.Y.Z-windows-portable.zip` 다운 → 압축 풀고 `Quota.exe` 더블클릭. 시스템 안 건드림, 지울 땐 파일 삭제
- **Linux**: `Quota-vX.Y.Z-linux-portable.tar.gz` 다운 → 풀고 `./Quota` 실행

처음 켜면 로그인 화면이 뜬다. 위젯이 사용량을 읽으려면 **Claude Code CLI**가 필요:

1. [claude.ai/install](https://claude.ai/install) 에서 Claude Code 설치
2. 터미널/PowerShell에서:
   ```
   claude
   ```
   실행 후 안에서:
   ```
   /login
   ```
   브라우저 OAuth로 로그인 (Pro/Max 구독 필요)
3. 위젯의 "로그인 완료 — 새로고침" 클릭

---

## 개발자 — 클론해서 빌드할 거면

### 사전 준비

**모든 OS 공통:**
- [Node.js](https://nodejs.org) LTS
- [Rust](https://rustup.rs) (rustup으로 설치)

**Windows 추가:**
- Visual Studio Build Tools 2022 + "C++ build tools" 워크로드 (MSVC 컴파일러용)

**Linux 추가:**
```
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev pkg-config
```

### 빌드 / 실행

```
git clone https://github.com/rendezvousgo/ai_widget.git
cd ai_widget
npm install
```

**개발 모드 (HMR로 바로 실행):**
```
npm run tauri dev
```

**프로덕션 빌드 (인스톨러 생성):**
```
npm run tauri build
```

빌드 결과물 위치:
- Windows: `src-tauri/target/release/bundle/msi/` 와 `bundle/nsis/`
- Linux: `src-tauri/target/release/bundle/appimage/` 와 `bundle/deb/`

---

## 자동 빌드 (GitHub Actions)

`v*` 태그 푸시하면 Windows + Linux 동시 빌드 후 Releases에 draft 업로드:

```
git tag v0.1.0
git push --tags
```

`.github/workflows/release.yml` 참고.

---

## 폴링 주기

- 프론트 자동 새로고침: 45초 (`src/App.svelte`)
- Rust 백엔드 캐시 TTL: 45초 (`src-tauri/src/oauth_usage.rs`)
- → 시간당 약 80회 Anthropic API 호출

조정하려면 두 값 같이 변경.
