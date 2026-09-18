#!/usr/bin/env python3
"""Build the single control showcase for a language and renderer."""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
from native_sdk import read_platform, c_command, rust_command

HERE = Path(__file__).resolve().parent
IN_REPO = (HERE.parent / 'crates/c-api').is_dir()
EXAMPLES = HERE.parent / 'examples' if IN_REPO else HERE / 'examples'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('language', choices=['c', 'rust', 'go', 'zig'])
parser.add_argument('--example', choices=['showcase', 'all'], default='showcase', help='all is a compatibility alias for showcase')
parser.add_argument('--backend', choices=['sdl3', 'raylib', 'opengl'], default='sdl3')
parser.add_argument('--sdk', type=Path, default=HERE.parent / 'target/native-ui-sdk' if IN_REPO else HERE)
parser.add_argument('--compiler', help='Compiler executable; otherwise use PATH')
parser.add_argument('--check', action='store_true', help='Build and run input/state checks without a window')
parser.add_argument('--smoke-test', action='store_true', help='Render eight hidden native frames (requires a display)')
parser.add_argument('--run', action='store_true', help='Open the example window after building')
args = parser.parse_args()
sdk = args.sdk.resolve()
if not (sdk / 'native_ui.h').is_file():
    parser.error('SDK missing; first run python3 tools/build-sdk.py')
target_info = read_platform(sdk)
windows = target_info.system == 'windows'
if windows and args.language in ('go', 'zig'):
    parser.error('The Windows build helper currently supports C and Rust; Go/Zig sources require a compatible Windows binding toolchain')
compiler_name = {'c': 'cl' if windows else 'cc', 'rust': 'cargo'}.get(args.language, args.language)
compiler = args.compiler or shutil.which(compiler_name)
if not compiler:
    parser.error(f'{compiler_name} missing; use --compiler /path/to/{compiler_name}')
compiler = os.path.abspath(compiler) if '/' in compiler or '\\' in compiler else compiler
mac = target_info.system == 'macos'
expected_system = {'darwin': 'macos', 'win32': 'windows', 'linux': 'linux'}.get(sys.platform)
if target_info.system != expected_system:
    parser.error('Use the SDK for the operating system you are building on')
# Unverified reference bindings still use the optional owning connector ABI.
link_sdk = sdk
if args.language in ('go', 'zig') and (sdk / 'standalone').is_dir():
    link_sdk = sdk / 'standalone'
library = link_sdk / target_info.runtime
if not library.is_file():
    parser.error(f'SDK library missing: {library}')
arch = target_info.architecture
arm = arch == 'arm64'
rpath = '@loader_path' if mac else '$ORIGIN'
env = os.environ.copy()
env.pop('NUI_EXAMPLE_MODE', None)
env['NUI_EXAMPLE_BACKEND'] = args.backend
if IN_REPO and args.language == 'c' and not env.get('CMAKE_PREFIX_PATH'):
    # Repository builds already have matching development packages from Cargo.
    # Installed/explicit host frameworks always take precedence.
    release = HERE.parent / 'target' / target_info.target / 'release'
    prefixes = {str(p.parent.parent.parent.parent) for pattern in
        ('build/sdl3-sys-*/out/lib/cmake/SDL3/SDL3Config.cmake',
         'build/raylib-sys-*/out/lib/cmake/raylib/raylib-config.cmake')
        for p in release.glob(pattern)}
    env['CMAKE_PREFIX_PATH'] = os.pathsep.join(sorted(prefixes))
source_dir = EXAMPLES / 'controls' / args.backend / args.language
def build_c(source, output, temporary):
    build = Path(temporary) / (output.stem + '-build')
    command = ['cmake', '-S', str(EXAMPLES / 'controls'), '-B', str(build),
        '-Dnative_ui_DIR=' + str(sdk / 'cmake'), '-DUI_BACKEND=' + args.backend,
        '-DUI_SOURCE=' + str(source), '-DUI_OUTPUT=' + str(output),
        '-DCMAKE_C_COMPILER=' + compiler]
    if mac:
        command += ['-DCMAKE_OSX_ARCHITECTURES=' + arch]
    subprocess.run(command, env=env, check=True)
    subprocess.run(['cmake', '--build', str(build), '--config', 'Release'], env=env, check=True)

if args.language == 'rust':
    # Native Rust examples deliberately use the host framework's ordinary API.
    manifest = source_dir / 'Cargo.toml'
    command = [compiler, 'build', '--manifest-path', str(manifest), '--target', target_info.target]
    subprocess.run(command, env=env, check=True)
    if args.check:
        subprocess.run([compiler, 'test', '--manifest-path', str(manifest), '--target', target_info.target], env=env, check=True)
    for enabled, mode in ((args.smoke_test, 'smoke'), (args.run, '')):
        if enabled:
            subprocess.run([compiler, 'run', '--manifest-path', str(manifest), '--target', target_info.target],
                env={**env, 'NUI_EXAMPLE_MODE': mode}, check=True)
    sys.exit(0)
if args.language in ('go', 'zig'):
    print('UNVERIFIED reference example: SDK-owned standalone connector; not the recommended borrowed integration.', flush=True)

names = ['showcase']
with tempfile.TemporaryDirectory(prefix='native-ui-example-') as temporary:
    if args.language == 'zig':
        target = ('aarch64' if arm else 'x86_64') + ('-macos' if mac else '-linux-gnu')
        translated = Path(temporary) / 'native_ui.zig'
        with translated.open('w') as f:
            subprocess.run([compiler, 'translate-c', '-target', target, '-lc', str(sdk / 'native_ui.h')], stdout=f, env=env, check=True)
    if args.language == 'go':
        env.update({'CGO_ENABLED': '1', 'GOARCH': 'arm64' if arm else 'amd64',
                    'GOOS': 'darwin' if mac else 'linux', 'GOTOOLCHAIN': 'local'})
        env['CGO_CFLAGS'] = shlex.join(['-I' + str(sdk), *(['-arch', arch] if mac else [])])
        env['CGO_LDFLAGS'] = shlex.join(['-L' + str(link_sdk), '-lnative_ui', '-Wl,-rpath,' + rpath, *(['-arch', arch] if mac else [])])
    for name in names:
        prefix = '' if args.backend == 'sdl3' else args.backend + '-'
        output = link_sdk / (f'{prefix}{args.language}-{name}' + target_info.executable_suffix)
        cwd = source_dir
        if args.language == 'go':
            command = [compiler, 'build', '-o', str(output), '.']
        elif args.language == 'zig':
            command = [compiler, 'build-exe', '-target', target, '-O', 'ReleaseSafe', '-lc',
                       '-L' + str(link_sdk), '-lnative_ui', '-rpath', '@executable_path' if mac else '$ORIGIN',
                       '--dep', 'native_ui', '-Mroot=' + str(source_dir / (name + '.zig')),
                       '-Mnative_ui=' + str(translated), '-femit-bin=' + str(output)]
        elif args.language == 'c':
            # Compile a lone copied source to guarantee that the example needs
            # only the SDK, never private headers from its original directory.
            source = Path(temporary) / (name + '.c')
            shutil.copyfile(source_dir / source.name, source)
            command = None
            build_c(source, output, temporary)
        else:
            command = rust_command(compiler, source_dir / (name + '.rs'), output, sdk, target_info)
        if command is not None:
            subprocess.run(command, cwd=cwd, env=env, check=True)
        test_output = output
        if args.language == 'c' and (args.check or args.smoke_test):
            # Tests own all synthetic input, assertions, and smoke-mode hooks.
            # The shipped sample itself is always compiled unmodified above.
            for header in ('support.h', name + '.h'):
                shutil.copyfile(source_dir / 'tests' / header, Path(temporary) / header)
            harness = Path(temporary) / 'test.c'
            harness.write_text(
                '#include "support.h"\n'
                '#define main example_main\n'
                f'#include "{name}.c"\n'
                '#undef main\n'
                f'#include "{name}.h"\n'
                'int main(void) {\n'
                '    const char *mode = getenv("NUI_EXAMPLE_MODE");\n'
                '    if (mode && strcmp(mode, "check") == 0) {\n'
                '        NuiUi *ui = nui_create(HTML, CSS, 940, 640);\n'
                '        require(ui != NULL);\n'
                '        check(ui);\n'
                '        ok(nui_destroy(ui));\n'
                '        return 0;\n'
                '    }\n'
                '    return example_main();\n'
                '}\n', encoding='utf-8')
            test_output = output.with_name(output.stem + '-tests' + output.suffix)
            build_c(harness, test_output, temporary)
        if args.check and args.language != 'c':
            test_output = output.with_name(output.stem + '-tests' + output.suffix)
            if args.language == 'rust':
                test_command = rust_command(compiler, source_dir / 'showcase.rs', test_output, sdk, target_info) + ['--test']
            elif args.language == 'go':
                test_command = [compiler, 'test', '-c', '-o', str(test_output), '.']
            else:
                test_command = [compiler, 'build-exe', '-target', target, '-O', 'ReleaseSafe', '-lc',
                    '-L' + str(link_sdk), '-lnative_ui', '-rpath', '@executable_path' if mac else '$ORIGIN',
                    '--dep', 'showcase', '-Mroot=' + str(source_dir / 'tests/checks.zig'),
                    '--dep', 'native_ui', '-Mshowcase=' + str(source_dir / 'showcase.zig'),
                    '-Mnative_ui=' + str(translated), '-femit-bin=' + str(test_output)]
            subprocess.run(test_command, cwd=cwd, env=env, check=True)
        for enabled, mode in ((args.check, 'check'), (args.smoke_test, 'smoke'), (args.run, '')):
            if enabled:
                executable = test_output if mode == 'check' or (mode == 'smoke' and args.language == 'c') else output
                subprocess.run([str(executable)], cwd=link_sdk, env={**env, 'NUI_EXAMPLE_MODE': mode}, check=True)
        print(f'Built {output}', flush=True)
