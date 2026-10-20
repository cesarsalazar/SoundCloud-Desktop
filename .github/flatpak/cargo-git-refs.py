import json
import re
import sys
import tomllib
from urllib.parse import parse_qsl, urlsplit

REF_KINDS = ("tag", "rev", "branch")


def canonical(url):
    return url.lower().rstrip("/").removesuffix(".git")


def lock_refs(lock_path):
    with open(lock_path, encoding="utf-8") as lock:
        sources = set(re.findall(r'^source = "git\+([^"#]+)', lock.read(), re.M))
    for source in sorted(sources):
        parts = urlsplit(source)
        ref = next(((kind, value) for kind, value in parse_qsl(parts.query) if kind in REF_KINDS), None)
        if ref:
            yield source, parts._replace(query="").geturl(), ref


def configured(config):
    return {
        (canonical(entry["git"]), kind, entry[kind])
        for entry in config.get("source", {}).values()
        if "git" in entry
        for kind in REF_KINDS
        if kind in entry
    }


def entry(source, url, kind, value):
    return f'\n[source."{source}"]\ngit = "{url}"\nreplace-with = "vendored-sources"\n{kind} = "{value}"\n'


def main(lock_path, sources_path):
    with open(sources_path, encoding="utf-8") as file:
        sources = json.load(file)
    config = next(s for s in sources if s.get("dest") == "cargo" and s.get("dest-filename") == "config")
    present = configured(tomllib.loads(config["contents"]))
    for source, url, (kind, value) in lock_refs(lock_path):
        if (canonical(url), kind, value) not in present:
            config["contents"] += entry(source, url, kind, value)
    with open(sources_path, "w", encoding="utf-8") as file:
        json.dump(sources, file, indent=4)


main(*sys.argv[1:3])
