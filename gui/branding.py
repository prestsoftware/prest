import platform_specific

with open(platform_specific.get_embedded_file_path('version.txt')) as f:
    VERSION = f.read().strip()

PREST_VERSION = f'Prest {VERSION}'
