import math
from threading import Thread
import time
import struct

import zmq


clock = 0


def flat_tone(frequency: float, sample_rate: int = 48000, n_samples: int = 2048) -> bytes:
    def next_value() -> float:
        global clock
        clock = clock + 1 % sample_rate
        val = math.sin((clock * frequency * 2.0 * math.pi) / sample_rate)
        return val
    floats = [next_value() for _ in range(n_samples)]
    # pack list of floars as big-endian bytes representing corresponding float32
    packed = b"".join([struct.pack(">f", v) for v in floats])
    return packed


def connect_and_send(frequency: float, channel: int = 0) -> None:
    print("sending %d on channel %d" % (frequency, channel))
    ctx = zmq.Context()
    socket = ctx.socket(zmq.PUB)
    socket.bind("ipc:///tmp/.psynth.%d" % channel)

    # manual sleep because zmq doesn't handle short-lived sockets as expected
    time.sleep(0.25)
    for _ in range(20):
        socket.send(flat_tone(frequency))
        time.sleep(1 / (48000 / 2048 + 10))


if __name__ == "__main__":
    freqs = [440, 1000, 1200]
    # freqs = [200]
    threads = [Thread(target=connect_and_send, args=(freq, i)) for i, freq in enumerate(freqs)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()