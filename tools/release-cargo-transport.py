#!/usr/bin/env python3
"""Hold one private Cargo boundary through preflight, intent and upload.

Usage: private JSON-lines protocol from release-cargo-upload.mjs; Linux only.
No token is accepted in preparation. The sole publisher owns attempt authorization.
"""
import ctypes
import fcntl
import hashlib
import json
import os
from pathlib import Path
import resource
import select
import shlex
import signal
import stat
import subprocess
import sys
import tempfile

PHASE = "initialization"
CREDENTIAL_RELEASED = False


def phase(name):
    global PHASE
    PHASE = name


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def emit(record):
    print(json.dumps({"status": "pass", **record}), flush=True)


def line(stream, timeout):
    require(select.select([stream], [], [], timeout)[0], "protocol timeout")
    data = stream.readline(1048577)
    require(data and len(data) <= 1048576 and data.endswith("\n"), "invalid protocol line")
    return json.loads(data)


def run(args, cwd, env, timeout=600):
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, timeout=timeout)
    if result.returncode != 0 and not CREDENTIAL_RELEASED:
        sys.stderr.write(result.stderr[-8192:].decode("utf8", errors="replace"))
    require(result.returncode == 0, "command failed")
    return result.stdout.decode("utf8")


def inventory(root):
    root = root.resolve()
    result = {}
    for path in sorted(root.rglob("*")):
        relative = str(path.relative_to(root))
        info = path.lstat()
        if stat.S_ISLNK(info.st_mode):
            require(not relative.startswith(".git/") and path.resolve().is_relative_to(root), "escaping or Git-metadata symlink")
            result[relative] = {"kind": "symlink", "target": os.readlink(path)}
        elif stat.S_ISREG(info.st_mode):
            result[relative] = {"kind": "file", "mode": stat.S_IMODE(info.st_mode), "sha256": sha(path.read_bytes()), "size": info.st_size}
        else:
            require(stat.S_ISDIR(info.st_mode), "nonregular snapshot input")
            result[relative] = {"kind": "directory", "mode": stat.S_IMODE(info.st_mode)}
    return result


def git(repo, env, *args):
    return run(["/usr/bin/git", "--no-replace-objects", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null", "-C", str(repo), *args], repo, env)


def inner():
    global CREDENTIAL_RELEASED
    phase("mounted-inventory")
    require(str(Path(__file__)) == "/tmp/release-helpers/transport.py", "inner entry requires private mounted helpers")
    policy = json.loads(Path("/tmp/release-helpers/policy.json").read_text())
    expected = json.loads(Path("/tmp/release-helpers/inventory.json").read_text())
    source = Path("/tmp/release-candidate")
    env = dict(os.environ)
    home = Path("/tmp/release-home")
    phase("cache-copy")
    for path in Path("/tmp/release-cache").rglob("*"):
        if path.is_file():
            destination = home / path.relative_to("/tmp/release-cache")
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(path.read_bytes())
    def verify():
        phase("source-inventory")
        observed = inventory(source)
        if observed != expected and not CREDENTIAL_RELEASED:
            difference = [key for key in sorted(set(expected) | set(observed)) if expected.get(key) != observed.get(key)]
            sys.stderr.write(json.dumps({"inventoryDifferences": [{"path": key, "expected": expected.get(key), "observed": observed.get(key)} for key in difference[:8]], "total": len(difference)}) + "\n")
        require(observed == expected, "mounted candidate differs")
        phase("candidate-head")
        require(git(source, env, "rev-parse", "HEAD").strip() == policy["candidateSha"], "candidate Git identity differs")
        phase("candidate-clean")
        require(not git(source, env, "status", "--porcelain", "--untracked-files=all").strip(), "candidate is dirty")
    verify()
    phase("git-integrity")
    git(source, env, "fsck", "--full", "--strict")
    phase("package-preflight")
    command = [policy["cargo"], "package", "--registry", policy["registry"], "--locked", "--no-verify", "--target-dir", "/tmp/release-target"]
    run(command, source, env)
    archive = Path("/tmp/release-target/package") / ("tkt-" + policy["version"] + ".crate")
    require(archive.stat().st_size == policy["size"] and sha(archive.read_bytes()) == policy["sha256"], "preflight differs from seal")
    verify()
    phase("intent-wait")
    emit({"phase": "prepared", "candidateSha": policy["candidateSha"], "sha256": policy["sha256"], "size": policy["size"], "inventorySha256": sha(json.dumps(expected, sort_keys=True).encode()), "files": sum(item["kind"] != "directory" for item in expected.values())})
    request = line(sys.stdin, 1800)
    if request == {"action": "close"}:
        return
    require(set(request) == {"action", "token"} and request["action"] == "publish", "unexpected attempt command")
    CREDENTIAL_RELEASED = True
    token = request["token"]
    require(isinstance(token, str) and 0 < len(token) <= 8192 and all(33 <= ord(char) <= 126 for char in token), "invalid credential")
    require(policy["registry"] != "release-probe" or token == "local-probe-only", "real credential forbidden for probe")
    verify()
    phase("cargo-publish")
    publish = [policy["cargo"], "publish", "--registry", policy["registry"], "--locked", "--no-verify", "--target-dir", "/tmp/release-target"]
    result = subprocess.run(publish, cwd=source, env={**env, "RELEASE_CARGO_TOKEN": token}, capture_output=True, timeout=600)
    # Registry observation owns recovery; never expose a server response containing credentials.
    require(result.returncode == 0, "Cargo attempt unconfirmed")
    verify()
    emit({"phase": "published", "exit": result.returncode, "candidateSha": policy["candidateSha"], "sha256": policy["sha256"]})


def outer():
    require(sys.platform == "linux" and len(sys.argv) == 1, "Linux protocol entry required")
    # Parent death must unwind temporary trees, not bypass their context managers.
    def terminate(signum, _frame):
        raise SystemExit(128 + signum)
    signal.signal(signal.SIGTERM, terminate)
    parent = os.getppid()
    require(parent != 1 and ctypes.CDLL(None, use_errno=True).prctl(1, signal.SIGTERM) == 0 and os.getppid() == parent, "parent lifecycle unavailable")
    policy = line(sys.stdin, 30)
    require(isinstance(policy, dict) and set(policy) == {"schema", "repo", "candidateSha", "name", "version", "sha256", "size", "cargo", "rustc", "node", "index", "registry"} and policy["schema"] == 1, "invalid preparation policy")
    env = {"PATH": "/usr/bin:/bin", "LANG": "C", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_OPTIONAL_LOCKS": "0", "GIT_NO_REPLACE_OBJECTS": "1", "CARGO_HTTP_PROXY": "", "RUSTC": policy["rustc"]}
    for key in ["cargo", "rustc", "node"]:
        path = Path(policy[key]).resolve()
        require(path.is_file() and path.parts[1] in {"usr", "bin", "sbin", "lib", "lib64", "home", "opt"}, "runtime outside supported trusted roots")
        policy[key] = str(path)
    require(run(["/usr/bin/bwrap", "--version"], "/", env).strip() == "bubblewrap 0.6.1", "unverified bubblewrap version")
    require(run([policy["cargo"], "--version"], "/", env).strip() == "cargo 1.99.1 (5f94df478 2026-08-27)", "unverified Cargo version")
    require(run([policy["rustc"], "--version"], "/", env).startswith("rustc 1.99.1 "), "unverified rustc version")
    descriptors = []
    child = None
    with tempfile.TemporaryDirectory(prefix="release-cargo-") as temporary:
        root = Path(temporary)
        template = root / "template"
        template.mkdir()
        source = root / "candidate"
        home = root / "cargo-home"
        home.mkdir()
        env.update({"HOME": str(home), "CARGO_HOME": str(home), "TMPDIR": str(root)})
        try:
            phase("candidate-clone")
            run(["/usr/bin/git", "clone", "--no-local", "--no-hardlinks", "--no-checkout", "--template", str(template), "--", policy["repo"], str(source)], root, env)
            git(source, env, "remote", "remove", "origin")
            entries = git(source, env, "ls-tree", "-rz", "--full-tree", policy["candidateSha"]).split("\0")
            require(all(not entry.startswith("160000 ") for entry in entries), "submodules are not sealed inputs")
            git(source, env, "checkout", "--detach", policy["candidateSha"])
            git(source, env, "fsck", "--full", "--strict")
            for path in source.rglob("*"):
                relative = str(path.relative_to(source))
                require(not relative.endswith((".cargo/config", ".cargo/config.toml")) and relative != ".gitmodules", "source Cargo configuration or submodule overrides are forbidden")
            require(not (source / ".git/objects/info/alternates").exists(), "external object store")
            expected = inventory(source)
            for ancestor in [source, *source.parents]:
                for name in [".cargo/config", ".cargo/config.toml"]:
                    require(not (ancestor / name).exists() and not (ancestor / name).is_symlink(), "ancestor Cargo configuration is forbidden")
            config = '[registry]\ndefault="' + policy["registry"] + '"\ncredential-provider=["/tmp/release-helpers/provider"]\n[registries.crates-io]\nprotocol="sparse"\n'
            if policy["registry"] == "release-probe":
                config += '[registries.release-probe]\nindex=' + json.dumps(policy["index"]) + '\ncredential-provider=["/tmp/release-helpers/provider"]\n'
            (home / "config.toml").write_text(config)
            # Fetch public locked dependencies before entering the namespace or accepting a token.
            phase("dependency-fetch")
            run([policy["cargo"], "fetch", "--locked"], source, env)
            require(inventory(source) == expected, "dependency fetch changed candidate inputs")
            phase("snapshot-sealing")
            helpers = root / "helpers"
            helpers.mkdir()
            (helpers / "transport.py").write_bytes(Path(__file__).read_bytes())
            (helpers / "provider.mjs").write_bytes(Path(__file__).with_name("release-cargo-credentials.mjs").read_bytes())
            (helpers / "provider").write_text("#!/bin/sh\nexec " + shlex.quote(policy["node"]) + " /tmp/release-helpers/provider.mjs \"$@\"\n")
            (helpers / "provider").chmod(0o755)
            (helpers / "policy.json").write_text(json.dumps(policy))
            (helpers / "inventory.json").write_text(json.dumps(expected, sort_keys=True))
            inputs = [(source, "/tmp/release-candidate"), (helpers, "/tmp/release-helpers")]
            files = []
            directories = {}
            symlinks = []
            for base, destination in inputs:
                for relative, item in inventory(base).items():
                    target = destination + "/" + relative
                    for parent in Path(target).parents:
                        if str(parent).startswith(destination):
                            directories.setdefault(str(parent), 0o755)
                    if item["kind"] == "directory":
                        directories[target] = item["mode"]
                    elif item["kind"] == "symlink":
                        symlinks.append((item["target"], target))
                    else:
                        files.append((base / relative, target, item))
            for folder in [home / "registry/cache", home / "registry/index"]:
                if folder.is_dir():
                    for path in sorted(folder.rglob("*")):
                        require(not path.is_symlink(), "cache symlink")
                        if path.is_file():
                            target = "/tmp/release-cache/" + str(path.relative_to(home))
                            for parent in Path(target).parents:
                                if str(parent).startswith("/tmp/release-cache"):
                                    directories.setdefault(str(parent), 0o755)
                            files.append((path, target, {"mode": 0o444, "sha256": sha(path.read_bytes()), "size": path.stat().st_size}))
            files.append((home / "config.toml", "/tmp/release-home/config.toml", {"mode": 0o444, "sha256": sha((home / "config.toml").read_bytes()), "size": (home / "config.toml").stat().st_size}))
            require(len(files) <= 10000 and sum(item["size"] for _, _, item in files) <= 256 * 1024 * 1024, "snapshot resource limit")
            soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
            needed = len(files) + 64
            if soft != resource.RLIM_INFINITY and soft < needed:
                require(hard == resource.RLIM_INFINITY or hard >= needed, "snapshot descriptor limit")
                resource.setrlimit(resource.RLIMIT_NOFILE, (needed, hard))
            args = ["--unshare-user", "--unshare-pid", "--unshare-ipc", "--unshare-uts", "--die-with-parent", "--new-session", "--cap-drop", "ALL", "--tmpfs", "/"]
            for directory in ["/usr", "/bin", "/sbin", "/lib", "/lib64", "/etc", "/home", "/opt", "/run"]:
                if Path(directory).exists():
                    args.extend(["--ro-bind", directory, directory])
            args.extend(["--proc", "/proc", "--dev", "/dev", "--tmpfs", "/tmp"])
            for directory in ["candidate", "helpers", "cache", "home", "target", "tmp"]:
                args.extend(["--tmpfs", "/tmp/release-" + directory])
            for directory, mode in sorted(directories.items()):
                args.extend(["--perms", format(mode, "o"), "--dir", directory])
            seals = fcntl.F_SEAL_WRITE | fcntl.F_SEAL_GROW | fcntl.F_SEAL_SHRINK | fcntl.F_SEAL_SEAL
            def sealed(data, mode=0o444):
                descriptor = os.memfd_create("release-input", os.MFD_ALLOW_SEALING | os.MFD_CLOEXEC)
                descriptors.append(descriptor)
                with os.fdopen(os.dup(descriptor), "wb") as stream:
                    stream.write(data)
                os.fchmod(descriptor, mode)
                fcntl.fcntl(descriptor, fcntl.F_ADD_SEALS, seals)
                require(fcntl.fcntl(descriptor, fcntl.F_GET_SEALS) & seals == seals and os.pread(descriptor, len(data), 0) == data, "input sealing failed")
                os.lseek(descriptor, 0, os.SEEK_SET)
                return descriptor
            for path, destination, item in files:
                data = path.read_bytes()
                require(len(data) == item["size"] and sha(data) == item["sha256"], "snapshot changed during capture")
                args.extend(["--perms", format(item["mode"], "o"), "--ro-bind-data", str(sealed(data, item["mode"])), destination])
            for target, destination in symlinks:
                args.extend(["--symlink", target, destination])
            for directory in ["/tmp/release-candidate", "/tmp/release-helpers", "/tmp/release-cache", "/tmp", "/"]:
                args.extend(["--remount-ro", directory])
            args.append("--clearenv")
            inner_env = {"PATH": "/usr/bin:/bin", "LANG": "C", "HOME": "/tmp/release-home", "CARGO_HOME": "/tmp/release-home", "TMPDIR": "/tmp/release-tmp", "RUSTC": policy["rustc"], "CARGO_HTTP_PROXY": "", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_OPTIONAL_LOCKS": "0", "GIT_NO_REPLACE_OBJECTS": "1", "RELEASE_CARGO_POLICY": "/tmp/release-helpers/policy.json"}
            for key, value in inner_env.items():
                args.extend(["--setenv", key, value])
            args.extend(["--chdir", "/tmp/release-candidate"])
            require(len(args) + 6 <= 9000, "bubblewrap argument limit")
            arg_fd = sealed(("\0".join(args) + "\0").encode())
            phase("namespace-setup")
            # v0.6.1 parses only options from --args; its command stays on real argv.
            child = subprocess.Popen(["/usr/bin/bwrap", "--args", str(arg_fd), "--", "/usr/bin/python3", "/tmp/release-helpers/transport.py", "--inner"], env=env, pass_fds=descriptors, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                prepared = line(child.stdout, 600)
            except (RuntimeError, ValueError) as error:
                if child.poll() is None:
                    child.kill()
                _, diagnostic = child.communicate(timeout=10)
                # The token callback cannot run until a prepared record is returned.
                sys.stderr.write(diagnostic[:8192])
                raise RuntimeError("private preflight process failed") from error
            if prepared.get("status") == "error":
                phase(prepared.get("phase", "private-preflight"))
                _, diagnostic = child.communicate(timeout=10)
                sys.stderr.write(diagnostic[:8192])
            require(prepared.get("status") == "pass" and prepared.get("phase") == "prepared", "private preflight failed")
            emit({key: value for key, value in prepared.items() if key != "status"})
            phase("intent-wait")
            request = line(sys.stdin, 1800)
            child.stdin.write(json.dumps(request) + "\n")
            child.stdin.flush()
            if request == {"action": "close"}:
                child.wait(timeout=10)
                return
            result = line(child.stdout, 660)
            if result.get("status") == "error":
                phase(result.get("phase", "cargo-publish"))
            require(result.get("status") == "pass" and result.get("phase") == "published", "Cargo attempt unconfirmed")
            child.wait(timeout=10)
            require(child.returncode == 0, "Cargo boundary exited unexpectedly")
            emit({key: value for key, value in result.items() if key != "status"})
        finally:
            if child is not None and child.poll() is None:
                child.kill()
                child.communicate()
            for descriptor in descriptors:
                os.close(descriptor)


if __name__ == "__main__":
    try:
        if sys.argv[1:] == ["--inner"]:
            inner()
        else:
            outer()
    except Exception:
        # This protocol boundary must not echo input, Cargo output or credentials.
        print(json.dumps({"status": "error", "phase": PHASE, "errors": ["private Cargo phase failed; inspect observed registry state before retrying"]}), flush=True)
        sys.exit(2)
