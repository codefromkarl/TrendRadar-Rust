#!/usr/bin/env python3
"""Measure the Python fixture pipeline baseline against the adjacent TrendRadar repo."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import platform
import shutil
import sys
import textwrap
import threading
import time
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PYTHON_REPO = REPO_ROOT.parent / "TrendRadar"
HOTLIST_FIXTURE = REPO_ROOT / "fixtures/system/fetch/hotlist-weibo.json"
RSS_FIXTURE = REPO_ROOT / "fixtures/system/fetch/rss-rust-blog.json"
DEFAULT_OUTPUT_JSON = REPO_ROOT / "target/python-benchmark-baseline/fixture_pipeline_minimal.json"


@dataclass
class BenchmarkSummary:
    profile: str
    python_cli_entry: str
    python_bridge_entry: str
    python_repo: str
    rust_fixture_config: str
    rust_hotlist_fixture: str
    rust_rss_fixture: str
    measured_runs: int
    warmup_runs: int
    elapsed_ns: list[int]
    min_ns: int
    max_ns: int
    median_ns: int
    mean_ns: int
    range_human: str
    median_human: str
    machine: str
    cpu: str
    os: str
    python_version: str
    measured_at: str
    measurement_tool: str
    notes: list[str]


class FixtureRequestHandler(BaseHTTPRequestHandler):
    hotlist_payload: str = ""
    rss_payload: str = ""

    def do_GET(self) -> None:  # noqa: N802
        parsed = urlparse(self.path)
        if parsed.path == "/api/s":
            query = parse_qs(parsed.query)
            source_id = query.get("id", [""])[0]
            if source_id != "weibo":
                self.send_error(404, "unknown hotlist source")
                return
            body = self.hotlist_payload.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if parsed.path == "/rss/rust-blog.xml":
            body = self.rss_payload.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/rss+xml; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        self.send_error(404, "unknown fixture path")

    def log_message(self, format: str, *args: Any) -> None:  # noqa: A003
        return


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Measure fixture_pipeline_minimal on the adjacent Python TrendRadar repo.",
    )
    parser.add_argument(
        "--python-repo",
        default=str(DEFAULT_PYTHON_REPO),
        help="Path to the adjacent Python TrendRadar repository.",
    )
    parser.add_argument(
        "--warmups",
        type=int,
        default=3,
        help="Warm-up runs before measuring.",
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=10,
        help="Measured runs.",
    )
    parser.add_argument(
        "--output-json",
        default=str(DEFAULT_OUTPUT_JSON),
        help="Where to write the benchmark summary JSON.",
    )
    return parser.parse_args()


def load_hotlist_items() -> list[dict[str, Any]]:
    rows = json.loads(HOTLIST_FIXTURE.read_text(encoding="utf-8"))
    items: list[dict[str, Any]] = []
    for index, row in enumerate(rows, start=1):
        slug = f"hotlist-{index}"
        items.append(
            {
                "title": row["title"],
                "url": f"https://example.invalid/{slug}",
                "mobileUrl": f"https://m.example.invalid/{slug}",
            }
        )
    return items


def load_rss_items() -> list[dict[str, Any]]:
    return json.loads(RSS_FIXTURE.read_text(encoding="utf-8"))


def build_rss_xml(rss_items: list[dict[str, Any]]) -> str:
    pub_date = datetime.now(UTC).strftime("%a, %d %b %Y %H:%M:%S GMT")
    items_xml = []
    for item in rss_items:
        summary = item.get("title", "")
        items_xml.append(
            textwrap.dedent(
                f"""\
                <item>
                  <title>{xml_escape(item["title"])}</title>
                  <link>{xml_escape(item["url"])}</link>
                  <description>{xml_escape(summary)}</description>
                  <pubDate>{pub_date}</pubDate>
                  <guid>{xml_escape(item["url"])}</guid>
                </item>
                """
            ).strip()
        )

    return textwrap.dedent(
        f"""\
        <?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <title>Rust Blog Fixture</title>
            <link>https://example.invalid/rust-blog</link>
            <description>TrendRadar Python benchmark fixture</description>
            {"".join(items_xml)}
          </channel>
        </rss>
        """
    ).strip()


def xml_escape(value: str) -> str:
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
    )


def start_fixture_server() -> tuple[ThreadingHTTPServer, threading.Thread, str]:
    hotlist_payload = json.dumps(
        {"status": "success", "items": load_hotlist_items()},
        ensure_ascii=False,
    )
    rss_payload = build_rss_xml(load_rss_items())

    FixtureRequestHandler.hotlist_payload = hotlist_payload
    FixtureRequestHandler.rss_payload = rss_payload

    server = ThreadingHTTPServer(("127.0.0.1", 0), FixtureRequestHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base_url = f"http://127.0.0.1:{server.server_port}"
    return server, thread, base_url


def ensure_import_path(python_repo: Path) -> None:
    python_repo_str = str(python_repo)
    if python_repo_str not in sys.path:
        sys.path.insert(0, python_repo_str)


def write_benchmark_config(workdir: Path, base_url: str) -> Path:
    config_dir = workdir / "config"
    config_dir.mkdir(parents=True, exist_ok=True)
    (config_dir / "frequency_words.txt").write_text("", encoding="utf-8")

    config_yaml = textwrap.dedent(
        f"""\
        app:
          timezone: "Asia/Shanghai"
          show_version_update: false

        schedule:
          enabled: false

        platforms:
          enabled: true
          sources:
            - id: "weibo"
              name: "微博"

        rss:
          enabled: true
          freshness_filter:
            enabled: false
            max_age_days: 0
          feeds:
            - id: "rust-blog"
              name: "Rust Blog"
              url: "{base_url}/rss/rust-blog.xml"
              enabled: true
              max_items: 50

        report:
          mode: "current"
          display_mode: "keyword"
          rank_threshold: 10
          sort_by_position_first: false
          max_news_per_keyword: 0

        notification:
          enabled: false

        filter:
          method: "keyword"

        ai:
          provider: ""

        ai_analysis:
          enabled: false

        ai_translation:
          enabled: false

        ai_filter:
          enabled: false

        storage:
          backend: "local"
          formats:
            sqlite: true
            txt: false
            html: true
          local:
            data_dir: "output"
            retention_days: 0

        advanced:
          debug: false
          version_check_url: ""
          configs_version_check_url: ""
          crawler:
            request_interval: 0
            use_proxy: false
            default_proxy: ""
          rss:
            request_interval: 0
            timeout: 5
            use_proxy: false
            proxy_url: ""
        """
    )
    config_path = config_dir / "config.yaml"
    config_path.write_text(config_yaml, encoding="utf-8")
    return config_path


@contextlib.contextmanager
def patched_environ(extra: dict[str, str]):
    original = {key: os.environ.get(key) for key in extra}
    os.environ.update(extra)
    try:
        yield
    finally:
        for key, old_value in original.items():
            if old_value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = old_value


@contextlib.contextmanager
def pushd(path: Path):
    previous = Path.cwd()
    os.chdir(path)
    try:
        yield
    finally:
        os.chdir(previous)


def reset_python_storage_singleton() -> None:
    from trendradar.storage import manager as storage_manager_module

    storage_manager_module._storage_manager = None  # type: ignore[attr-defined]


def run_python_pipeline_once(python_repo: Path, base_url: str, run_dir: Path) -> int:
    ensure_import_path(python_repo)
    config_path = write_benchmark_config(run_dir, base_url)

    import trendradar.__main__ as trendradar_main
    from trendradar.crawler.fetcher import DataFetcher

    reset_python_storage_singleton()
    DataFetcher.DEFAULT_API_URL = f"{base_url}/api/s"
    trendradar_main.NewsAnalyzer._should_open_browser = lambda self: False

    capture = io.StringIO()
    started = time.perf_counter_ns()
    try:
        with (
            pushd(run_dir),
            patched_environ(
                {
                    "CONFIG_PATH": str(config_path),
                    "FREQUENCY_WORDS_PATH": str(run_dir / "config/frequency_words.txt"),
                    "DOCKER_CONTAINER": "true",
                }
            ),
            contextlib.redirect_stdout(capture),
            contextlib.redirect_stderr(capture),
        ):
            config = trendradar_main.load_config(str(config_path))
            analyzer = trendradar_main.NewsAnalyzer(config=config)
            analyzer.run()
    except Exception as exc:  # pragma: no cover - failure path only
        output = capture.getvalue()
        raise RuntimeError(
            f"Python benchmark pipeline failed in {run_dir}\n{output}"
        ) from exc
    finally:
        reset_python_storage_singleton()
    return time.perf_counter_ns() - started


def benchmark_python_pipeline(
    python_repo: Path,
    base_url: str,
    warmups: int,
    runs: int,
) -> list[int]:
    benchmark_root = REPO_ROOT / "target/python-benchmark-baseline/runs"
    benchmark_root.mkdir(parents=True, exist_ok=True)

    total_runs = warmups + runs
    elapsed: list[int] = []
    for index in range(total_runs):
        run_dir = benchmark_root / f"run-{index:02d}"
        shutil.rmtree(run_dir, ignore_errors=True)
        run_dir.mkdir(parents=True, exist_ok=True)
        duration_ns = run_python_pipeline_once(python_repo, base_url, run_dir)
        if index >= warmups:
            elapsed.append(duration_ns)
    return elapsed


def cpu_model() -> str:
    cpuinfo = Path("/proc/cpuinfo")
    if cpuinfo.exists():
        for line in cpuinfo.read_text(encoding="utf-8").splitlines():
            if line.lower().startswith("model name"):
                _, value = line.split(":", 1)
                return value.strip()
    return platform.processor() or "unknown"


def format_ns_range(min_ns: int, max_ns: int) -> str:
    if max_ns < 1_000_000:
        return f"{min_ns / 1_000:.2f} µs ~ {max_ns / 1_000:.2f} µs"
    return f"{min_ns / 1_000_000:.2f} ms ~ {max_ns / 1_000_000:.2f} ms"


def format_ns_value(value_ns: int) -> str:
    if value_ns < 1_000_000:
        return f"{value_ns / 1_000:.2f} µs"
    return f"{value_ns / 1_000_000:.2f} ms"


def median_ns(values: list[int]) -> int:
    ordered = sorted(values)
    middle = len(ordered) // 2
    if len(ordered) % 2 == 1:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle]) // 2


def mean_ns(values: list[int]) -> int:
    return sum(values) // len(values)


def build_summary(args: argparse.Namespace, elapsed: list[int]) -> BenchmarkSummary:
    min_value = min(elapsed)
    max_value = max(elapsed)
    median_value = median_ns(elapsed)
    mean_value = mean_ns(elapsed)
    measured_at = datetime.now().astimezone().isoformat(timespec="seconds")
    return BenchmarkSummary(
        profile="fixture_pipeline_minimal",
        python_cli_entry="python -m trendradar",
        python_bridge_entry="trendradar.__main__.NewsAnalyzer.run (patched DataFetcher.DEFAULT_API_URL)",
        python_repo=str(Path(args.python_repo).resolve()),
        rust_fixture_config="fixtures/system/config/minimal-valid.json",
        rust_hotlist_fixture="fixtures/system/fetch/hotlist-weibo.json",
        rust_rss_fixture="fixtures/system/fetch/rss-rust-blog.json",
        measured_runs=args.runs,
        warmup_runs=args.warmups,
        elapsed_ns=elapsed,
        min_ns=min_value,
        max_ns=max_value,
        median_ns=median_value,
        mean_ns=mean_value,
        range_human=format_ns_range(min_value, max_value),
        median_human=format_ns_value(median_value),
        machine=platform.node(),
        cpu=cpu_model(),
        os=platform.platform(),
        python_version=platform.python_version(),
        measured_at=measured_at,
        measurement_tool="time.perf_counter_ns with warm-up runs; local fixture HTTP server + NewsAnalyzer.run bridge",
        notes=[
            "The benchmark fixes the real CLI entry at trendradar.__main__.py / python -m trendradar.",
            "Hotlist API is locally bridged because the Python CLI does not expose a configurable API base URL.",
            "RSS feed is served from a local fixture HTTP endpoint to avoid network noise.",
            "Each iteration uses a fresh output directory to avoid historical data affecting current-mode timing.",
        ],
    )


def write_summary(path: Path, summary: BenchmarkSummary) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(asdict(summary), ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def main() -> int:
    args = parse_args()
    python_repo = Path(args.python_repo).resolve()
    if not python_repo.exists():
        raise SystemExit(f"Python repo not found: {python_repo}")

    server, thread, base_url = start_fixture_server()
    try:
        elapsed = benchmark_python_pipeline(
            python_repo=python_repo,
            base_url=base_url,
            warmups=args.warmups,
            runs=args.runs,
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=1)

    summary = build_summary(args, elapsed)
    output_path = Path(args.output_json).resolve()
    write_summary(output_path, summary)

    print(json.dumps(asdict(summary), ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
