import hashlib
import json
import pathlib
import selectors
import subprocess
import time
import sys
from urllib.parse import urlparse

root = pathlib.Path(sys.argv[1]).resolve()
archive = root / 'vibe-acp.tar.gz'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == '96276b6d7cc49473e48a79f95c397a05dc7f8b1d5c459893562e0dd533112074'
name = 'firetower-mistral-linux-acp'
script = '''set -eu
! command -v python
! command -v python3
! command -v uv
mkdir -p /tmp/staging /tmp/agents/mistral-vibe/3.1.0 /tmp/private-home
chmod 700 /tmp/private-home
tar xzf /evidence/vibe-acp.tar.gz -C /tmp/staging
mv /tmp/staging /tmp/agents/mistral-vibe/3.1.0/bin
export VIBE_HOME=/tmp/private-home VIBE_TEST_DISABLE_KEYRING=1 VIBE_ENABLE_TELEMETRY=false
unset MISTRAL_API_KEY
/tmp/agents/mistral-vibe/3.1.0/bin/vibe-acp --version >&2
exec /tmp/agents/mistral-vibe/3.1.0/bin/vibe-acp
'''
started = time.monotonic()
with (root / 'stderr.log').open('w') as errors:
    process = subprocess.Popen(['docker', 'run', '--name', name, '--label', 'com.firetower.validation=mistral-acp', '-i', '--mount', f'type=bind,source={root},target=/evidence,readonly', '--entrypoint', '/bin/sh', 'oven/bun:1.3.14-debian', '-c', script], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=errors, text=True, bufsize=1)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    def rpc(identifier, method, params):
        process.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': identifier, 'method': method, 'params': params}) + '\n')
        process.stdin.flush()
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            if selector.select(timeout=max(0, deadline-time.monotonic())):
                line = process.stdout.readline()
                assert line, 'CLI stopped before response'
                message = json.loads(line)
                if message.get('id') == identifier and ('result' in message or 'error' in message):
                    assert 'error' not in message, f'ACP error code: {message["error"].get("code")}'
                    return message['result']
        raise TimeoutError(method)
    try:
        initialized = rpc(1, 'initialize', {'protocolVersion': 1, 'clientInfo': {'name': 'firetower-linux-smoke', 'version': '1'}, 'clientCapabilities': {'_meta': {'browser-auth-delegated': True}}})
        assert initialized['protocolVersion'] == 1
        assert any(method['id'] == 'browser-auth-delegated' for method in initialized['authMethods'])
        result = rpc(2, 'authenticate', {'methodId': 'browser-auth-delegated', 'action': 'start'})
        delegated = result['_meta']['browser-auth-delegated']
        url = urlparse(delegated['signInUrl'])
        assert url.scheme == 'https' and url.hostname == 'console.mistral.ai'
        assert delegated['attemptId'] and delegated['expiresAt']
        report = {'image': 'oven/bun:1.3.14-debian', 'platform': 'linux/aarch64 glibc 2.41', 'vibe_version': '3.1.0', 'archive_sha256': hashlib.sha256(archive.read_bytes()).hexdigest(), 'host_python_or_uv': False, 'staging_rename': True, 'acp_protocol': 1, 'delegated_auth_link_host': url.hostname, 'elapsed_seconds': round(time.monotonic()-started, 2), 'account_approved': False, 'model_turn': False, 'scope': 'direct official CLI smoke; not Firetower Linux worker installation acceptance'}
        print(json.dumps(report, indent=2))
        (root / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    finally:
        subprocess.run(['docker', 'stop', '--time', '2', name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=15)
        process.communicate(timeout=15)
