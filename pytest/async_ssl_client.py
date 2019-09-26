#!/usr/bin/env python

from __future__ import print_function

import asyncio
import argparse
import threading
import time
import capnp
import socket
import ssl
import sys

sys.path.append("../schema")

import echo_capnp


def parse_args():
  parser = argparse.ArgumentParser(usage='Connects to the Example thread server \
at the given address and does some RPCs')
  parser.add_argument("host", help="HOST:PORT")

  return parser.parse_args()


async def myreader(client, reader):
  while True:
    data = await reader.read(4096)
    client.write(data)


async def mywriter(client, writer):
  while True:
    data = await client.read(4096)
    writer.write(data.tobytes())


async def main(host):
  host = host.split(':')
  addr = host[0]
  port = host[1]

  # Setup SSL context
  ctx = ssl.SSLContext()

  # Handle both IPv4 and IPv6 cases
  try:
    print("Try IPv4")
    reader, writer = await asyncio.open_connection(
      addr, port,
      ssl=ctx,
    )
  except:
    print("Try IPv6")
    reader, writer = await asyncio.open_connection(
      addr, port,
      ssl=ctx,
      family=socket.AF_INET6
    )

  # Start TwoPartyClient using TwoWayPipe (takes no arguments in this mode)
  client = capnp.TwoPartyClient()
  cap = client.bootstrap().cast_as(echo_capnp.Echo)

  # Assemble reader and writer tasks, run in the background
  coroutines = [myreader(client, reader), mywriter(client, writer)]
  asyncio.gather(*coroutines, return_exceptions=True)

  # Run blocking tasks
  print('main: {}'.format(time.time()))
  print(await cap.echo("test").a_wait())
  print('main: {}'.format(time.time()))
  print(await cap.echo("test1").a_wait())
  print('main: {}'.format(time.time()))
  print(await cap.echo("test2").a_wait())
  print('main: {}'.format(time.time()))
  print(await cap.echo("test3").a_wait())

if __name__ == '__main__':
  # Using asyncio.run hits an asyncio ssl bug
  # https://bugs.python.org/issue36709
  #asyncio.run(main(parse_args().host), loop=loop, debug=True)
  loop = asyncio.get_event_loop()
  loop.run_until_complete(main(parse_args().host))
