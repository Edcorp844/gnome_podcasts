desktop_file = i18n.merge_file(
  input: 'org.flame.podcasts.desktop.in',
  output: application_id + '.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications',
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = i18n.merge_file(
  input: 'org.flame.podcasts.metainfo.xml.in',
  output: application_id + '.metainfo.xml',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'metainfo',
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
test(
  'Validate appstream file',
  appstreamcli,
  args: ['validate', '--no-net', '--explain', appstream_file],
)

# --- gschema: substitute the app id so dev/release builds each get
# --- a schema whose <schema id="..."> matches the actual running app id ---
gschema_conf = configuration_data()
gschema_conf.set('app-id', application_id)

gschema_file = configure_file(
  input: 'org.flame.podcasts.gschema.xml.in',
  output: application_id + '.gschema.xml',
  configuration: gschema_conf,
)

install_data(gschema_file, install_dir: get_option('datadir') / 'glib-2.0' / 'schemas')

compile_schemas = find_program('glib-compile-schemas', required: false, disabler: true)
test(
  'Validate schema file',
  compile_schemas,
  args: ['--strict', '--dry-run', meson.current_build_dir()],
)

service_conf = configuration_data()
service_conf.set('bindir', get_option('prefix') / get_option('bindir'))
configure_file(
  input: 'org.flame.podcasts.service.in',
  output: application_id + '.service',
  configuration: service_conf,
  install_dir: get_option('datadir') / 'dbus-1' / 'services',
)

subdir('icons')