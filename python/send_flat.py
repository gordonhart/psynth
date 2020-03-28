import math
import time
import struct

import zmq


def flat_tone(frequency: float, sample_rate: int = 48000, length: int = 8192) -> bytes:
    clock = 0
    def next_value() -> float:
        nonlocal clock
        clock = clock + 1 % sample_rate
        val = (clock * frequency * 2.0 * math.pi) % sample_rate
        return val
    floats = [next_value() for _ in range(length // 4)]
    packed = b"".join([struct.pack(">f", v) for v in floats])
    return packed


if __name__ == "__main__":
    ctx = zmq.Context()
    socket = ctx.socket(zmq.PUB)
    socket.bind("ipc:///tmp/.psynth.0")

    # manual sleep because zmq doesn't handle short-lived sockets as expected
    time.sleep(0.25)
    socket.send(flat_tone(440))
