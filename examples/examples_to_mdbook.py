# /// script
# requires-python = ">=3.12"
# dependencies = [
# ]
# ///
import base64
import json
import os
import sys
import tomllib
from pathlib import Path
from typing import NotRequired, TypedDict

PERFETTO_SCRIPT = """
<script>
function openTraceInPerfetto(base64TraceData, traceTitle = 'Trace') {
  const PERFETTO_ORIGIN = 'https://ui.perfetto.dev';

  // Decode base64 to ArrayBuffer
  const binaryString = atob(base64TraceData);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  const arrayBuffer = bytes.buffer;

  // Open Perfetto UI
  const win = window.open(PERFETTO_ORIGIN);
  if (!win) {
    console.error('Popup blocked. Please allow popups for this site.');
    return;
  }

  // PING/PONG handshake
  const timer = setInterval(() => win.postMessage('PING', PERFETTO_ORIGIN), 50);

  const onMessageHandler = (evt) => {
    if (evt.data !== 'PONG') return;

    // UI is ready, send trace data
    clearInterval(timer);
    window.removeEventListener('message', onMessageHandler);

    win.postMessage({
      perfetto: {
        buffer: arrayBuffer,
        title: traceTitle,
        fileName: `${traceTitle}.trace`
      }
    }, PERFETTO_ORIGIN);
  };

  window.addEventListener('message', onMessageHandler);
}
</script>
"""


class ExampleToml(TypedDict):
    name: str
    example_file: str
    text_start: str
    trace_file: NotRequired[str]
    annotated_trace_file: NotRequired[str]
    sort_key: int


def find_examples() -> list[tuple[Path, ExampleToml]]:
    script_dir = Path(os.path.abspath(os.path.dirname(__file__)))
    folders = [
        folder for folder in script_dir.iterdir()
        if folder.is_dir() and (folder / "example.toml").is_file()
    ]

    examples = []
    for folder in folders:
        config_path = folder / "example.toml"
        with config_path.open("rb") as f:
            examples.append((folder, tomllib.load(f)))

    return examples


def render_example(example_folder: Path, info: ExampleToml, strict: bool, parent_chapter_num: list[int], parent_names: list[str], example_num: int) -> dict:
    content = render_example_content(example_folder, info, strict)

    return {
        "Chapter": {
            "name": info["name"],
            "content": content,
            "number": parent_chapter_num + [example_num],
            "sub_items": [],
            "path": "gen_examples/" + info["name"].lower().replace(" ", "_") + "_example",
            "source_path": "TODO",
            "parent_names": parent_names
        }
    }


def include_perfetto_trace(trace: Path, name: str) -> tuple[str, str]:
    with open(trace, "rb") as f:
        trace_data = f.read()
    base64_trace = base64.b64encode(trace_data).decode('ascii')
    data = f"<script>const TRACE_{name} = '{base64_trace}';</script>"
    link = f"<a href=\"javascript: openTraceInPerfetto(TRACE_{name}, 'CircumSpect Example')\">here</a> "
    return (data, link)


def render_example_content(example_folder: Path, info: ExampleToml, strict: bool) -> str:
    page = [f"# {info['name']}"]
    page += [""]
    page += [info['text_start']]
    page += [""]

    # Include open-in-perfetto script:
    page += [PERFETTO_SCRIPT]
    page += [""]

    missing_traces = []

    trace_file_link = None
    annotated_trace_file_link = None

    script_dir = Path(os.path.abspath(os.path.dirname(__file__)))
    shared_out_dir = (script_dir / "out").resolve()

    if "trace_file" in info:
        trace_path = shared_out_dir / info["trace_file"]
        if trace_path.exists():
            data, link = include_perfetto_trace(trace_path, "raw")
            page += [data]
            page += [""]
            trace_file_link = link
        else:
            trace_file_link = "here (missing)"
            missing_traces += ["trace_file"]

    if "annotated_trace_file" in info:
        trace_path = shared_out_dir / info["annotated_trace_file"]
        if trace_path.exists():
            data, link = include_perfetto_trace(trace_path, "annotated")
            page += [data]
            page += [""]
            annotated_trace_file_link = link
        else:
            annotated_trace_file_link = "here (missing)"
            missing_traces += ["annotated_trace_file"]

    match (trace_file_link, annotated_trace_file_link):
        case (str(), None):
            page += [""]
            page += [f"Click {trace_file_link} to view the output of this example in Perfetto."]
        case (str(), str()):
            page += [""]
            page += [f"Click {trace_file_link} to view the raw output and {annotated_trace_file_link} to view the annotated output of this example in Perfetto."]
        case (None, None):
            pass
        case (None, str()):
            print(f"Example only has annotated trace {info['name']}", file=sys.stderr)
            sys.exit(1)

    if missing_traces:
        if strict:
            print(f"Missing traces for example {info['name']}: {missing_traces}", file=sys.stderr)
            sys.exit(1)
        page += ["> [!CAUTION]"]
        page += ["> Example trace outputs are missing: {", ".join(missing_traces)}"]
        page += []

    example_path = example_folder.joinpath(info['example_file'])
    with open(example_path, "r") as example_file:
        example_code = example_file.read()

    page += ["```verilog"]
    page += [example_code]
    page += ["```"]

    return "\n".join(page)


if __name__ == '__main__':
    # We support all backends:
    if len(sys.argv) > 1:
        if sys.argv[1] == "supports":
            sys.exit(0)

    # Potentially enable "complete"/"ci" mode.
    # This causes the generation to fail if any input file is missing. Enabled
    # in CI/for deployment, but not enabled by default during development to
    # make it easy to view the docs.
    strict_mode = 'CSPECT_DOCS_STRICT' in os.environ

    # load mdbook context + book:
    context, book = json.load(sys.stdin)

    # Find "Examples" chapter:
    ch_num = None
    ch_name = None
    ch_sub_items = None
    for section in book["sections"]:
        if "Chapter" not in section:
            continue

        chapter = section["Chapter"]

        if chapter["name"] != "Examples":
            continue

        ch_num = chapter["number"]
        ch_name = chapter["parent_names"] + [chapter["name"]]
        chapter["sub_items"] = []
        ch_sub_items = chapter["sub_items"]  # type: list | None
        break

    if ch_num is None or ch_name is None or ch_sub_items is None:
        print("Could not find 'Examples' chapter!", file=sys.stderr)
        print(f"found ch_num: {ch_num}", file=sys.stderr)
        print(f"found name: {ch_name}", file=sys.stderr)
        print(f"found ch_sub_items: {ch_sub_items}", file=sys.stderr)
        sys.exit(1)

    # Find examples and render to pages:
    examples = find_examples()
    examples.sort(key=lambda item: item[1]["sort_key"])
    for num, (folder, info) in enumerate(examples):
        ch_sub_items.append(render_example(folder, info, strict_mode, ch_num, ch_name, num))

    print(json.dumps(book))
