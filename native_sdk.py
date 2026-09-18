"""Platform metadata and compiler commands shared by SDK build tools."""
from dataclasses import dataclass
import json
from pathlib import Path
import platform
import subprocess
import sys


@dataclass(frozen=True)
class SdkPlatform:
    target: str
    system: str
    architecture: str
    runtime: str
    source_runtime: str
    import_library: str | None = None

    @property
    def asset_name(self):
        return f'native-ui-{self.system}-{self.architecture}'

    @property
    def executable_suffix(self):
        return '.exe' if self.system == 'windows' else ''


PLATFORMS = {
    'aarch64-apple-darwin': SdkPlatform('aarch64-apple-darwin', 'macos', 'arm64', 'libnative_ui.dylib', 'libnative_ui_c.dylib'),
    'x86_64-apple-darwin': SdkPlatform('x86_64-apple-darwin', 'macos', 'x86_64', 'libnative_ui.dylib', 'libnative_ui_c.dylib'),
    'x86_64-unknown-linux-gnu': SdkPlatform('x86_64-unknown-linux-gnu', 'linux', 'x86_64', 'libnative_ui.so', 'libnative_ui_c.so'),
    # Keep the DLL's original name: it is embedded in the linker-generated import library.
    'x86_64-pc-windows-msvc': SdkPlatform('x86_64-pc-windows-msvc', 'windows', 'x86_64', 'native_ui_c.dll', 'native_ui_c.dll', 'native_ui.lib'),
}


def rust_host():
    return next(line.removeprefix('host: ') for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))


def read_platform(sdk: Path):
    metadata = sdk / 'sdk.json'
    if metadata.is_file():
        return PLATFORMS[json.loads(metadata.read_text(encoding='utf-8'))['target']]
    # Compatibility with SDKs produced before the metadata file was introduced.
    if sys.platform == 'darwin':
        arch = subprocess.check_output(['lipo', '-archs', str(sdk / 'libnative_ui.dylib')], text=True).strip()
        return PLATFORMS[('aarch64' if arch == 'arm64' else arch) + '-apple-darwin']
    if sys.platform == 'win32':
        return PLATFORMS['x86_64-pc-windows-msvc']
    return PLATFORMS[platform.machine() + '-unknown-linux-gnu']


def c_command(compiler, language, source, output, sdk, target):
    """Use MSVC on Windows and the platform C/C++ compiler elsewhere."""
    if target.system == 'windows':
        return [compiler, '/nologo', '/utf-8', '/std:c11' if language == 'c' else '/std:c++17',
                '/TC' if language == 'c' else '/TP', '/O2', '/W3', '/WX', '/D_CRT_SECURE_NO_WARNINGS',
                *(['/EHsc'] if language == 'cpp' else []), str(source), '/I' + str(sdk),
                '/Fo' + str(output.with_suffix('.obj')), '/Fe' + str(output),
                '/link', '/LIBPATH:' + str(sdk), target.import_library]
    mac = target.system == 'macos'
    return [compiler, *(['-arch', target.architecture] if mac else []),
            '-x', 'c++' if language == 'cpp' else 'c',
            '-std=c++17' if language == 'cpp' else '-std=c11',
            '-O2', '-Wall', '-Wextra', '-Werror', str(source), '-I' + str(sdk),
            '-L' + str(sdk), '-lnative_ui', '-Wl,-rpath,' + ('@loader_path' if mac else '$ORIGIN'),
            '-o', str(output)]


def rust_command(compiler, source, output, sdk, target):
    command = [compiler, '--edition=2024', '--target', target.target, str(source), '-L', str(sdk)]
    if target.system != 'windows':
        command += ['-C', 'link-arg=-Wl,-rpath,' + ('@loader_path' if target.system == 'macos' else '$ORIGIN')]
    return [*command, '-o', str(output)]


def archive_sdk(sdk: Path, destination: Path):
    """Ship libraries and consumer sources, excluding build outputs and test executables."""
    import hashlib
    import zipfile
    target = read_platform(sdk)
    destination.mkdir(parents=True, exist_ok=True)
    archive = destination / (target.asset_name + '.zip')
    if archive.exists():
        raise FileExistsError(archive)
    roots = [sdk / name for name in (target.runtime, 'native_ui.h', 'sdk.json', 'README.md',
             'LANGUAGES.md', 'NOTICES.txt', 'build-language.py', 'native_sdk.py', 'examples', 'cmake')]
    for name in ('native_ui_renderer.h', 'native_ui_raylib.h', 'verification.json', 'Cargo.toml', 'Cargo.lock', 'crates'):
        if (sdk / name).exists():
            roots.append(sdk / name)
    for name in (target.runtime, *([target.import_library] if target.import_library else [])):
        if (sdk / 'standalone' / name).is_file():
            roots.append(sdk / 'standalone' / name)
    if (sdk / 'themes').is_dir():
        roots.append(sdk / 'themes')
    if (sdk / 'docs').is_dir():
        roots.append(sdk / 'docs')
    if (sdk / 'OPENGL.md').is_file():
        roots.append(sdk / 'OPENGL.md')
    if (sdk / 'native_ui_sdl3.h').is_file():
        roots.append(sdk / 'native_ui_sdl3.h')
    if (sdk / 'native_ui_themes.h').is_file():
        roots.append(sdk / 'native_ui_themes.h')
    if target.import_library:
        roots.append(sdk / target.import_library)
    for root in roots:
        if not root.exists():
            raise FileNotFoundError(root)
    with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=9) as bundle:
        for root in roots:
            paths = sorted(root.rglob('*')) if root.is_dir() else [root]
            for path in paths:
                if not path.is_file() or '__pycache__' in path.parts or '.zig-cache' in path.parts:
                    continue
                bundle.write(path, Path(target.asset_name) / path.relative_to(sdk))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix('.zip.sha256').write_text(f'{digest}  {archive.name}\n', encoding='utf-8')
    return archive
