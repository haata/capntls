import os
import subprocess
import time

examples_dir = os.path.join(os.path.dirname(__file__))

def run_subprocesses(address, client):
    server = subprocess.Popen(['cargo', 'run', 'server', address], cwd=os.path.join(examples_dir, '..'))
    time.sleep(5)  # Give the server some small amount of time to start listening
    client = subprocess.Popen([os.path.join(examples_dir, client), address])

    ret = client.wait()
    server.kill()
    assert ret == 0


def test_ssl_async_example():
    address = 'localhost:36435'
    client = 'async_ssl_client.py'
    run_subprocesses(address, client)


def test_ssl_reconnecting_async_example():
    address = 'localhost:36436'
    client = 'async_reconnecting_ssl_client.py'
    run_subprocesses(address, client)
