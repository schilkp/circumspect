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
class Example:
    name: str
    folder: Path
    files: list[Path]
    out_dir: Path
    extra_verilator_flags: list[str]
    trace_file_name: str
    annotated_trace_file_name: str | None
    annotation_cmd: list[str] | None


@dataclass
class ExampleResult:
    name: str
    ok: bool
    note: str | None


def load_examples(script_dir: Path, requested_tests: list[str]) -> list[Example]:
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

        trace_file = cfg["trace_file"]

        annotation_cmd = cfg.get("annotation_cmd", None)

        annotated_trace_file = cfg.get("annotated_trace_file", None)

        tbs.append(Example(
            name=folder.name,
            folder=folder.resolve(),
            files=files,
            out_dir=out_dir,
            extra_verilator_flags=extra_verilator_flags,
            trace_file_name=trace_file,
            annotated_trace_file_name=annotated_trace_file,
            annotation_cmd=annotation_cmd
        ))

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


def verilator_run_examples(examples: list[Example], trace: bool) -> list[ExampleResult]:
    script_dir = Path(os.path.abspath(os.path.dirname(__file__)))
    common_cc_path = script_dir / "common_verilator_top.cc"
    shared_out_dir = script_dir / "out"
    shared_out_dir.mkdir(exist_ok=True)

    # Compile DPI library first
    dpi_success, dpi_info = compile_cspect(script_dir)
    if not dpi_success:
        print(f"{Color.ERR}Failed to compile DPI library:{Color.END}\n{dpi_info}")
        return [ExampleResult(name=example.name, ok=False, note="DPI library compilation failed") for example in examples]

    results = []

    compiled_tests = []

    print()
    print("Compiling examples..")
    for example in examples:
        (name, _, success, info) = verilator_build_example(
            example, common_cc_path, dpi_info)
        if success:
            print(f"  {name}: {Color.OK}OK{Color.END}")
            compiled_tests.append((info, example))
        else:
            print(f"  {name}: {Color.ERR}FAIL{Color.END}\n{info}")
            results.append(ExampleResult(
                name=example.name, ok=False, note="Failed to compile"))

    print()
    print("Running examples..")
    with concurrent.futures.ThreadPoolExecutor() as executor:
        run_results = list(executor.map(lambda nb:
                                        verilator_run_example(*nb, shared_out_dir=shared_out_dir, trace=trace), compiled_tests))

    for name, ok, output in run_results:
        status = f"{Color.OK}PASS{Color.END}" if ok else f"{Color.ERR}FAIL{Color.END}"
        print(f"  {name}: {status}")
        if not ok:
            print(f"    Output/Error:\n{output.strip()}")
        results.append(ExampleResult(name=name, ok=ok,
                       note=None if ok else "Example failed"))

    return results


def verilator_build_example(example: Example, common_cc_path: Path, dpi_lib_path: str):
    obj_dir = example.out_dir / "obj_dir"

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
        *example.extra_verilator_flags,
        *example.files,
        str(common_cc_path),
        "--Mdir", str(obj_dir)
    ]

    verilate_result = subprocess.run(
        verilator_cmd, cwd=example.folder, capture_output=True, text=True
    )
    (example.out_dir / "verilate.stdout").write_text(verilate_result.stdout)
    (example.out_dir / "verilate.stderr").write_text(verilate_result.stderr)
    if verilate_result.returncode != 0:
        return (example.name, example.out_dir, False, f"Verilator error:\n{verilate_result.stderr}")

    # Run make to build binary
    make_cmd = ["make", "-C", str(obj_dir), "-f",
                f"V{TOP_NAME}.mk", "-j", str(os.cpu_count())]
    make_result = subprocess.run(
        make_cmd, cwd=example.folder, capture_output=True, text=True
    )
    (example.out_dir / "make.stdout").write_text(make_result.stdout)
    (example.out_dir / "make.stderr").write_text(make_result.stderr)
    if make_result.returncode != 0:
        return (example.name, example.out_dir, False, f"Make error:\n{make_result.stderr}")

    # Path to test binary
    binary = obj_dir / f"V{TOP_NAME}"
    return (example.name, example.out_dir, True, str(binary))


def verilator_run_example(binary_path, example: Example, shared_out_dir: Path, trace=False):
    log = ""
    try:

        # Run Verilated Binary:
        env = os.environ.copy()
        if trace:
            env["VERILATOR_TRACE"] = "1"

        verilator_result = subprocess.run(
            [str(binary_path)],
            capture_output=True,
            text=True,
            cwd=example.out_dir,
            env=env
        )
        (example.out_dir / "verilator.stdout").write_text(verilator_result.stdout)
        (example.out_dir / "verilator.stderr").write_text(verilator_result.stderr)
        log += verilator_result.stdout
        log += verilator_result.stderr

        if verilator_result.returncode != 0:
            raise Exception(f"Verilator Exit Code: {verilator_result.returncode}")

        # Grab trace file:
        trace_file_src = example.out_dir / example.trace_file_name
        trace_file_dest = shared_out_dir / example.trace_file_name
        if not trace_file_src.exists():
            raise Exception(f"Trace file {trace_file_src} does not exist")
        shutil.copy(trace_file_src, trace_file_dest)

        # Annotate
        if example.annotation_cmd:
            annotate_result = subprocess.run(
                example.annotation_cmd,
                capture_output=True,
                text=True,
                cwd=example.folder,
                env=env
            )
            (example.out_dir / "annotate.stdout").write_text(annotate_result.stdout)
            (example.out_dir / "annotate.stderr").write_text(annotate_result.stderr)
            log += annotate_result.stdout
            log += annotate_result.stderr

            if annotate_result.returncode != 0:
                raise Exception(f"Annotate Exit Code: {annotate_result.returncode}")

        # Grab annotated file:
        if example.annotated_trace_file_name:
            trace_file_src = example.out_dir / example.annotated_trace_file_name
            trace_file_dest = shared_out_dir / example.annotated_trace_file_name
            if not trace_file_src.exists():
                raise Exception(f"Annotated trace file {trace_file_src} does not exist")
            shutil.copy(trace_file_src, trace_file_dest)

        return (example.name, True, log)

    except Exception as e:
        return (example.name, False, log + str(e))

# ===----------------------------------------------------------------------=== #
# Utils
# ===----------------------------------------------------------------------=== #


def clean(examples: list[Example]):
    print("Cleaning...")
    for example in examples:
        if example.out_dir.exists():
            shutil.rmtree(example.out_dir)


def print_test_summary(results: list[ExampleResult]) -> int:

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


def run_all_examples():
    script_dir = Path(os.path.abspath(os.path.dirname(__file__)))
    examples = load_examples(script_dir, [])
    results = verilator_run_examples(examples, False)
    exit(print_test_summary(results))


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
    examples = load_examples(script_dir, args.tests)

    if args.clean:
        clean(examples)
    else:
        results = verilator_run_examples(examples, args.trace)
        exit(print_test_summary(results))


if __name__ == "__main__":
    main()
