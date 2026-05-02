Name:           snotes
Version:        0.1.0
Release:        1%{?dist}
Summary:        Linux-native handwriting & annotation app
License:        GPL-3.0-or-later
URL:            https://github.com/SONUVERMA11/SNotes
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.75
BuildRequires:  cargo
BuildRequires:  gtk4-devel >= 4.12
BuildRequires:  libadwaita-devel >= 1.4
BuildRequires:  libinput-devel
BuildRequires:  sqlite-devel
BuildRequires:  pkgconfig
BuildRequires:  clang

Requires:       gtk4 >= 4.12
Requires:       libadwaita >= 1.4
Requires:       libinput
Requires:       sqlite

Recommends:     tesseract

%description
S Notes is a powerful handwriting and annotation application for Linux,
designed for students, professionals, and creatives who use stylus tablets.

Features include Bézier-based ink rendering with pressure sensitivity,
PDF import and annotation, multiple tool types (pen, brush, pencil,
marker, highlighter), shape recognition, multi-layer support with
page templates, and cloud sync via WebDAV/Nextcloud.

%prep
%autosetup

%build
cargo build --release -p snotes-gtk -p snotes-cli -p snotes-sync

%install
install -Dm755 target/release/snotes-gtk %{buildroot}%{_bindir}/snotes-gtk
install -Dm755 target/release/snotes-cli %{buildroot}%{_bindir}/snotes-cli
install -Dm755 target/release/snotes-sync %{buildroot}%{_bindir}/snotes-sync
install -Dm644 data/org.snotes.App.desktop %{buildroot}%{_datadir}/applications/org.snotes.App.desktop
install -Dm644 data/org.snotes.App.metainfo.xml %{buildroot}%{_datadir}/metainfo/org.snotes.App.metainfo.xml

%files
%license LICENSE
%doc README.md
%{_bindir}/snotes-gtk
%{_bindir}/snotes-cli
%{_bindir}/snotes-sync
%{_datadir}/applications/org.snotes.App.desktop
%{_datadir}/metainfo/org.snotes.App.metainfo.xml

%changelog
* Sat May 03 2026 Sonu Verma <https://github.com/SONUVERMA11> - 0.1.0-1
- Initial release
