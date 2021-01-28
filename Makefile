# TODO: create MSI installer
# 	- See MSITools https://wiki.gnome.org/msitools/HowTo/CreateMSI
# 	- https://github.com/GNOME/msitools
# 	- https://gitlab.gnome.org/GNOME/gedit/-/blob/gnome-3-32/win32/make-gedit-installer
# TODO: Add icon
# 	- See optional extras from https://gtk-rs.org/docs-src/tutorial/cross

MINGW_PREFIX=/mingw64
RELEASE_DIR=./build_win32
WINDOWS10_THEME_REPO_BASE=../
APP_RELEASE_DIR=./target/release
_wixdir="/c/Program Files (x86)/WiX Toolset v3.8"

# Skipped DLL files (to reduce install size). These are found using tools like Dependency walker
SKIP_DLLS = edit.dll libasprintf-0.dll libatomic-1.dll libcairo-script-interpreter-2.dll \
	libcharset-1.dll libcrypto-1_1-x64.dll libdeflate.dll libgailutil-3-0.dll libgccjit-0.dll \
	libgettextlib-0-19-8-1.dll libgettextpo-0.dll libgettextsrc-0-19-8-1.dll \
	libgfortran-5.dll libgif-7.dll libgirepository-1.0-1.dll libgladeui-2-13.dll libgmp-10.dll \
	libgmpxx-4.dll libgnarl-10.dll libgnat-10.dll libgnutls-30.dll libgnutlsxx-28.dll libgomp-1.dll \
	libharfbuzz-gobject-0.dll libharfbuzz-icu-0.dll libharfbuzz-subset-0.dll libhistory8.dll \
	libhogweed-6.dll libidn2-0.dll libjpeg-8.dll libjson-glib-1.0-0.dll liblzma-5.dll liblzo2-2.dll \
	libmpdec++-2.dll libmpdec-2.dll libnettle-8.dll libobjc-4.dll libp11-kit-0.dll libpcre16-0.dll \
	libpcre32-0.dll libpcrecpp-0.dll libpcreposix-0.dll libpkgconf-3.dll libproxy-1.dll libpython3.8.dll \
	libquadmath-0.dll libreadline8.dll librsvg-2-2.dll libsqlite3-0.dll libssl-1_1-x64.dll libssp-0.dll \
	libsystre-0.dll libtasn1-6.dll libtermcap-0.dll libtiff-5.dll libtiffxx-5.dll libtre-5.dll \
	libturbojpeg.dll libunistring-2.dll libwebp-7.dll libwebpdecoder-3.dll libwebpdemux-2.dll \
	libwebpmux-3.dll libxml2-2.dll libxxhash.dll libzstd.dll tcl86.dll tk86.dll

_arch=$(shell uname -m)
_date=$(shell date +'%Y%m%d')
_version=$(shell git describe --tags --abbrev=0)

APP_ZIP=visol-${_version}-${_arch}-${_date}.zip
APP_MSI=visol-${_version}-${_arch}-${_date}.msi

all: compile bundle_gtk copy_app copy_app_resources zip

# Still TODO
wix:
	# TODO: check resources/*.wix, resources/COPYING.rtf, resources/*.bmp, resources/*.svg
	# TODO: install MSITools or Wix Toolset
	"${_wixdir}/bin/heat.exe" dir ${RELEASE_DIR} -gg -dr INSTALLDIR -cg binaries -sfrag -sreg -srd -suid -template fragment -out binaries.wxs
	"${_wixdir}/bin/candle.exe" -arch x64 visol.wxs binaries.wxs
	"${_wixdir}/bin/light.exe" -ext WixUtilExtension -ext WixUIExtension visol.wixobj binaries.wixobj -o "${APP_MSI}"

zip:
	cd ${RELEASE_DIR} && zip -r ../${APP_ZIP} *

compile:
	cargo build --release

bundle_gtk: make_release_dirs install_theme basefiles

make_release_dirs:
	mkdir -p ${RELEASE_DIR}/etc/gtk-3.0
	mkdir -p ${RELEASE_DIR}/share/themes/Windows10/gtk-3.0
	mkdir -p ${RELEASE_DIR}/share/icons/hicolor
	mkdir -p ${RELEASE_DIR}/share/icons/Adwaita
	mkdir -p ${RELEASE_DIR}/share/glib-2.0/schemas
	mkdir -p ${RELEASE_DIR}/lib/gdk-pixbuf-2.0
	mkdir -p ${RELEASE_DIR}/lib/gtk-3.0
	mkdir -p ${RELEASE_DIR}/resources/shots

install_theme: copy_theme copy_icons ${RELEASE_DIR}/etc/gtk-3.0/settings.ini

pull_win10_theme:
	cd ${WINDOWS10_THEME_REPO_BASE} && git clone https://github.com/B00merang-Project/Windows-10.git

copy_theme: make_release_dirs
	cd ${WINDOWS10_THEME_REPO_BASE}/Windows-10 && git pull
	cp -R ${WINDOWS10_THEME_REPO_BASE}/Windows-10/gtk-3.20/* ${RELEASE_DIR}/share/themes/Windows10/gtk-3.0/

copy_icons: make_release_dirs
	cp -R ${MINGW_PREFIX}/share/icons/hicolor ${RELEASE_DIR}/share/icons
	cp -R ${MINGW_PREFIX}/share/icons/Adwaita/24x24 ${RELEASE_DIR}/share/icons/Adwaita
	cp -R ${MINGW_PREFIX}/share/icons/Adwaita/scalable ${RELEASE_DIR}/share/icons/Adwaita
	cp ${MINGW_PREFIX}/share/icons/Adwaita/index.theme ${RELEASE_DIR}/share/icons/Adwaita/

${RELEASE_DIR}/etc/gtk-3.0/settings.ini: make_release_dirs
	cp -R ${MINGW_PREFIX}/etc/gtk-3.0 ${RELEASE_DIR}/etc/gtk-3.0
	@echo "[Settings]" > $@
	@echo "gtk-theme-name=Windows10" >> $@
	@echo "gtk-font-name=Segoe UI 9" >> $@
	@echo "gtk-xft-rgba=rgb" >> $@

basefiles: glib_schemas glib_libs

glib_schemas: make_release_dirs
	glib-compile-schemas ${MINGW_PREFIX}/share/glib-2.0/schemas
	cp ${MINGW_PREFIX}/share/glib-2.0/schemas/gschemas.compiled ${RELEASE_DIR}/share/glib-2.0/schemas

glib_libs: make_release_dirs
	@find ${MINGW_PREFIX}/bin -maxdepth 1 -type f -name '*.dll' $(shell printf "! -name %s " ${SKIP_DLLS}) -exec cp "{}" ${RELEASE_DIR}/ \;
	# cp ${MINGW_PREFIX}/bin/*.dll ${RELEASE_DIR}/
	cp -R ${MINGW_PREFIX}/lib/gdk-pixbuf-2.0 ${RELEASE_DIR}/lib/
	cp -R ${MINGW_PREFIX}/lib/gtk-3.0 ${RELEASE_DIR}/lib/
	cp ${MINGW_PREFIX}/bin/gdbus.exe ${RELEASE_DIR}/

copy_app:
	cp ${APP_RELEASE_DIR}/*.exe ${RELEASE_DIR}/
	strip ${RELEASE_DIR}/*.exe
	# find ${RELEASE_DIR} -name *.dll | xargs strip
  	# find ${RELEASE_DIR} -name *.exe | xargs strip

copy_app_resources:
	cp -R ./src/data ${RELEASE_DIR}/
	cp -R ./src/res ${RELEASE_DIR}/
	cp -R ./resources/shots ${RELEASE_DIR}/resources/
	cp ./README.md ${RELEASE_DIR}/
	cp ./COPYING.txt ${RELEASE_DIR}/

install_msys2:
	pacman -Syu --needed --noconfirm mingw-w64-x86_64-toolchain base-devel mingw-w64-x86_64-adwaita-icon-theme mingw-w64-x86_64-gtk3 mingw-w64-x86_64-librsvg
	# pacman -Syu --needed --noconfirm mingw-w64-x86_64-glade
	# pacman -Syu --needed --noconfirm mingw-w64-x86_64-python3-gobject
