# --- Scalable app icon (single, named after application_id) ---
scalable_dir = 'hicolor' / 'scalable' / 'apps'
install_data(
  scalable_dir / ('@0@.svg').format('org.flame.podcasts'),
  install_dir: get_option('datadir') / 'icons' / scalable_dir
)

# --- Symbolic icons (auto-discovered: any *-symbolic.svg in this dir) ---
symbolic_dir = 'hicolor' / 'symbolic' / 'apps'

symbolic_icons = run_command(
  'find',
  meson.current_source_dir() / symbolic_dir,
  '-maxdepth', '1',
  '-name', '*-symbolic.svg',
  check: true
).stdout().strip().split('\n')

foreach icon_path : symbolic_icons
  if icon_path != ''
    install_data(
      icon_path,
      install_dir: get_option('datadir') / 'icons' / symbolic_dir
    )
  endif
endforeach