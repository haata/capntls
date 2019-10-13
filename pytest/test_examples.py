import os
import socket
import subprocess
import sys
import time

examples_dir = os.path.join(os.path.dirname(__file__))

def run_subprocesses(address, client):
    server = subprocess.Popen(['cargo', 'run', 'server', address], cwd=os.path.join(examples_dir, '..'))

    # Wait for server to start
    addr, port = address.split(':')
    retries = 20
    while True:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        result = sock.connect_ex((addr, int(port)))
        if result == 0:
            break
        sock = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
        result = sock.connect_ex((addr, int(port)))
        if result == 0:
            break
        # Give the server some small amount of time to start listening
        time.sleep(1)
        retries -= 1
        if retries == 0:
            assert False, "Timed out waiting for server to start"
    client = subprocess.Popen([sys.executable, os.path.join(examples_dir, client), address])

    ret = client.wait()
    server.kill()
    assert ret == 0, "Client did not return 0"


def test_ssl_async_example():
    address = 'localhost:36435'
    client = 'async_ssl_client.py'
    run_subprocesses(address, client)


def test_ssl_reconnecting_async_example():
    address = 'localhost:36436'
    client = 'async_reconnecting_ssl_client.py'
    run_subprocesses(address, client)
