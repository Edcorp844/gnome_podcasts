# 1. Populate config.rs from config.rs.in
global_conf = configuration_data()
global_conf.set_quoted('APP_ID', application_id)
global_conf.set_quoted('VERSION', xpodcasts_version + version_suffix)
global_conf.set_quoted('LOCALEDIR', xpodcasts_localedir)
global_conf.set_quoted('PKGDATADIR', xpodcasts_pkgdatadir)
global_conf.set_quoted('GETTEXT_PACKAGE', application_id)

config_rs = configure_file(
  input: 'config.rs.in',
  output: 'config.rs',
  configuration: global_conf,
)

# Copy generated config.rs to source dir for rust-analyzer / IDEs
run_command(
  'cp',
  config_rs,
  meson.current_source_dir() / 'config.rs',
  check: false,
)

# 2. Find Cargo executable
cargo = find_program('cargo', required: true)

# Determine build subfolder based on release vs debug
if get_option('buildtype') == 'release' or profile == 'release'
  cargo_target_dir = meson.project_build_root() / 'target' / 'release'
else
  cargo_target_dir = meson.project_build_root() / 'target' / 'debug'
endif

# 3. Custom target for Cargo build
cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: 'xpodcasts',
  install: true,
  install_dir: xpodcasts_bindir,
  console: true,
  env: cargo_env,
  command: [
    'sh', '-c',
    '@0@ build --manifest-path @1@ @2@ && cp @3@/@4@ @OUTPUT@'.format(
      cargo.full_path(),
      meson.project_source_root() / 'Cargo.toml',
      ' '.join(cargo_options),
      cargo_target_dir,
      meson.project_name(),
    )
  ],
)