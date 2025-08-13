# /// script
# requires-python = ">=3.12"
# dependencies = [
# ]
# ///
import argparse
import concurrent.futures
import os
import shutil
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path

TOP_NAME = "top"


class Color:
    OK = '\033[92m'
    WARN = '\033[93m'
    ERR = '\033[91m'
    INFO = '\033[96m'
    END = '\33[0m'


@dataclass
class Testbench:
    name: str
    folder: Path
    files: list[Path]
    out_dir: Path
    extra_verilator_flags: list[str]


@dataclass
class TestbenchResult:
    name: str
    ok: bool
    note: str | None


def load_testbenches(script_dir: Path, requested_tests: list[str]) -> list[Testbench]:
    # Find all subfolders containing example.toml
    folders = [
        folder for folder in script_dir.iterdir()
        if folder.is_dir() and (folder / "example.toml").is_file()
    ]

    # Select/filter if specific tests are requested via CLI:
    if len(requested_tests) > 0:
        wanted = set(requested_tests)
        found = set(f.name for f in folders)
        missing = wanted - found
        if missing:
            print(
                f"{Color.ERR}ERROR: Test(s) not found:{Color.END} {', '.join(sorted(missing))}")
            exit(2)
        folders = [f for f in folders if f.name in wanted]

    # Load TBs:
    tbs = []
    for folder in folders:
        config_path = folder / "example.toml"
        with config_path.open("rb") as f:
            cfg = tomllib.load(f)
        files = [(folder / x).resolve() for x in cfg["files"]]
        extra_verilator_flags = cfg.get("extra_verilator_flags", [])

        out_dir = folder / "out"
        out_dir.mkdir(exist_ok=True)

        tbs.append(Testbench(
            name=folder.name,
            folder=folder.resolve(),
            files=files,
            out_dir=out_dir,
            extra_verilator_flags=extra_verilator_flags)
        )

    return tbs

# ===----------------------------------------------------------------------=== #
# CircumSpect
# ===----------------------------------------------------------------------=== #


def compile_cspect(script_dir: Path) -> tuple[bool, str]:
    """Compile the shared DPI library"""
    parent_dir = script_dir.parent
    dpi_lib_path = parent_dir / "target" / "release" / "libcspect.a"

    print("Compiling shared DPI library...")
    cargo_cmd = ["cargo", "build", "--release", "--package", "cspect"]

    try:
        result = subprocess.run(
            cargo_cmd,
            cwd=parent_dir,
            capture_output=True,
            text=True
        )

        if result.returncode != 0:
            return (False, f"Cargo build failed:\n{result.stderr}")

        if not dpi_lib_path.exists():
            return (False, f"Expected library not found at: {dpi_lib_path}")

        print(f"  DPI library: {Color.OK}OK{Color.END}")
        return (True, str(dpi_lib_path))

    except Exception as e:
        return (False, f"Failed to run cargo: {str(e)}")

# ===----------------------------------------------------------------------=== #
# Verilator
# ===----------------------------------------------------------------------=== #


def verilator_run_tbs(tbs: list[Testbench], trace: bool) -> list[TestbenchResult]:
    script_dir = Path(os.path.abspath(os.path.dirname(__file__)))
    common_cc_path = script_dir / "common_verilator_top.cc"

    # Compile DPI library first
    dpi_success, dpi_info = compile_cspect(script_dir)
    if not dpi_success:
        print(f"{Color.ERR}Failed to compile DPI library:{Color.END}\n{dpi_info}")
        return [TestbenchResult(name=tb.name, ok=False, note="DPI library compilation failed") for tb in tbs]

    results = []

    compiled_tests = []

    print()
    print("Compiling testbenches..")
    for tb in tbs:
        (name, out_dir, success, info) = verilator_compile_testbench(
            tb, common_cc_path, dpi_info)
        if success:
            print(f"  {name}: {Color.OK}OK{Color.END}")
            compiled_tests.append((name, out_dir, info))
        else:
            print(f"  {name}: {Color.ERR}FAIL{Color.END}\n{info}")
            results.append(TestbenchResult(
                name=tb.name, ok=False, note="Failed to compile"))

    print()
    print("Running testbenches..")
    with concurrent.futures.ThreadPoolExecutor() as executor:
        run_results = list(executor.map(lambda nb:
                                        verilator_run_test_binary(
                                            *nb, trace=trace),
                                        compiled_tests))

    for name, ok, output in run_results:
        status = f"{Color.OK}PASS{Color.END}" if ok else f"{Color.ERR}FAIL{Color.END}"
        print(f"  {name}: {status}")
        if not ok:
            print(f"    Output/Error:\n{output.strip()}")
        results.append(TestbenchResult(name=name, ok=ok,
                       note=None if ok else "TB failed"))

    return results


def verilator_compile_testbench(tb: Testbench, common_cc_path: Path, dpi_lib_path: str):
    obj_dir = tb.out_dir / "obj_dir"

    # We always clean-build, since verilator does not re-link on DPI
    # lib changes.
    if obj_dir.exists():
        shutil.rmtree(obj_dir)

    verilator_cmd = [
        "verilator",
        "--cc",
        "--exe",
        "--top-module", TOP_NAME,
        "--timing",
        "--assert",
        "--trace-fst",
        "--trace-structs",
        "--trace-threads", "1",
        "--threads", "1",
        "-CFLAGS", "-O3",
        "-LDFLAGS", dpi_lib_path,
        *tb.extra_verilator_flags,
        *tb.files,
        str(common_cc_path),
        "--Mdir", str(obj_dir)
    ]

    verilate_result = subprocess.run(
        verilator_cmd, cwd=tb.folder, capture_output=True, text=True
    )
    (tb.out_dir / "verilate.stdout").write_text(verilate_result.stdout)
    (tb.out_dir / "verilate.stderr").write_text(verilate_result.stderr)
    if verilate_result.returncode != 0:
        return (tb.name, tb.out_dir, False, f"Verilator error:\n{verilate_result.stderr}")

    # Run make to build binary
    make_cmd = ["make", "-C", str(obj_dir), "-f",
                f"V{TOP_NAME}.mk", "-j", str(os.cpu_count())]
    make_result = subprocess.run(
        make_cmd, cwd=tb.folder, capture_output=True, text=True
    )
    (tb.out_dir / "make.stdout").write_text(make_result.stdout)
    (tb.out_dir / "make.stderr").write_text(make_result.stderr)
    if make_result.returncode != 0:
        return (tb.name, tb.out_dir, False, f"Make error:\n{make_result.stderr}")

    # Path to test binary
    binary = obj_dir / f"V{TOP_NAME}"
    return (tb.name, tb.out_dir, True, str(binary))


def verilator_run_test_binary(name, out_path, binary_path, trace=False):
    try:
        env = os.environ.copy()
        if trace:
            env["VERILATOR_TRACE"] = "1"
        result = subprocess.run(
            [str(binary_path)],
            capture_output=True,
            text=True,
            cwd=out_path,
            env=env
        )
        (out_path / "tb.stdout").write_text(result.stdout)
        (out_path / "tb.stderr").write_text(result.stderr)
        success = result.returncode == 0
        return (name, success, result.stdout + result.stderr)
    except Exception as e:
        return (name, False, str(e))

# ===----------------------------------------------------------------------=== #
# Utils
# ===----------------------------------------------------------------------=== #


def clean(tbs: list[Testbench]):
    print("Cleaning...")
    for tb in tbs:
        if tb.out_dir.exists():
            shutil.rmtree(tb.out_dir)


def print_test_summary(results: list[TestbenchResult]) -> int:

    print()
    print("Summary:")

    passes = [r for r in results if r.ok]
    fails = [r for r in results if not r.ok]

    for result in passes:
        print(f"  {result.name}: {Color.OK}PASS{Color.END}")
    for result in fails:
        note = ('(' + result.note + ')') if result.note else ''
        print(f"  {result.name}: {Color.ERR}FAIL{Color.END} {note}")

    if len(fails) != 0:
        print()
        print(f"{Color.ERR}:({Color.END}")
        return 1

    return 0

# ===----------------------------------------------------------------------=== #
# Main
# ===----------------------------------------------------------------------=== #


def main():
    parser = argparse.ArgumentParser(description="Run component tests.")
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Delete all generated files instead of building."
    )
    parser.add_argument(
        "--trace",
        action="store_true",
        help="Enable VERILATOR_TRACE=1 for test binary runs"
    )
    parser.add_argument(
        "tests",
        nargs="*",
        help="List of test folder names to run (default: run all)",
    )
    args = parser.parse_args()

    script_dir = Path(os.path.abspath(os.path.dirname(__file__)))
    tbs = load_testbenches(script_dir, args.tests)

    if args.clean:
        clean(tbs)
    else:
        results = verilator_run_tbs(tbs, args.trace)
        exit(print_test_summary(results))


if __name__ == "__main__":
    main()
