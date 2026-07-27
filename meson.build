project(
  'xpodcasts',
  'rust',
  version: '0.1.0',
  meson_version: '>= 1.0.0',
  default_options: ['warning_level=2', 'werror=false'],
)

# 1. System Dependencies
sqlite_dep = dependency('sqlite3', version: '>= 3.20')
openssl_dep = dependency('openssl', version: '>= 1.0')
dbus_dep = dependency('dbus-1')
gtk_dep = dependency('gtk4', version: '>= 4.15')
adw_dep = dependency('libadwaita-1', version: '>= 1.6')
gst_dep = dependency('gstreamer-1.0', version: '>= 1.22')
gst_play_dep = dependency('gstreamer-play-1.0', version: '>= 1.22')

# 2. Application Variables & Profile Handling
xpodcasts_version = meson.project_version()
version_suffix = ''

profile = get_option('profile')
if profile == 'development'
  application_id = 'org.flame.podcasts.Devel'
  version_suffix = '-dev'
  cargo_options = []
elif profile == 'release' or get_option('buildtype') == 'release'
  application_id = 'org.flame.podcasts'
  cargo_options = ['--release']
else
  application_id = 'org.flame.podcasts'
  cargo_options = []
endif

# 3. Directory Locations
prefix = get_option('prefix')
xpodcasts_bindir = prefix / get_option('bindir')
xpodcasts_localedir = prefix / get_option('localedir')
xpodcasts_pkgdatadir = prefix / get_option('datadir') / application_id

# 4. Configuration Data for src/config.rs.in
conf = configuration_data()
conf.set_quoted('APP_ID', application_id)
conf.set_quoted('VERSION', xpodcasts_version + version_suffix)
conf.set_quoted('LOCALEDIR', xpodcasts_localedir)
conf.set_quoted('PKGDATADIR', xpodcasts_pkgdatadir)
conf.set_quoted('RESOURCEFILE', xpodcasts_pkgdatadir / 'resources.gresource')

configure_file(
  input: 'src/config.rs.in',
  output: 'config.rs',
  configuration: conf,
)

# 5. Global Cargo Environment Setup
cargo_env = environment()
cargo_env.set('CARGO_TARGET_DIR', meson.project_build_root() / 'target')
cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-cache')

i18n = import('i18n')
gnome = import('gnome')

# Subdirectories
subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)