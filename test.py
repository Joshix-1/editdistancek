#!/usr/bin/env python3
import string, sys, time

from random import Random

from editdistancek_rs import distance as our_distance, distance_unbounded as our_unbounded
from rapidfuzz.distance.Levenshtein import distance as rf_distance

CHARACTERS = (
    f"{string.hexdigits}"
    "𓅳𓆜𓆾𓇆𓆦𓆗𓆣𓆁𓅰"
    "ৄ৬ਊઊଳஜ௫ಙೱൔ"
    "§¼ƝɸʬЖ¥©¶±"
    "äöüß"
)

def assert_eq(value: object, should: object, message: object) -> None:
    if value != should:
        raise AssertionError(f"{value!r} != {should!r} ({message})")

def random_string(random: Random, length: int) -> str:
    return "".join(random.choice(CHARACTERS) for _ in range(length))

class Timer:
    ns: int = 0

    def __enter__(self) -> None:
        self.start = time.perf_counter_ns()

    def __exit__(self, *_) -> None:
        took = time.perf_counter_ns() - self.start
        self.ns += took
        del self.start

def test(min_length: int, bounded: bool) -> None:
    random = Random(min_length)

    our_timer = Timer()
    rf_timer = Timer()

    for i in range(0, 10_000):
        word1 = random_string(random, min_length + random.randrange(0, i // 10 + 1))
        word2 = random_string(random, min_length + random.randrange(0, i // 10 + 1))

        if bounded:
            k = int(0.7 * len(word1))

            with our_timer:
                our = our_distance(word1, word2, k=k)

            with rf_timer:
                rf = rf_distance(word1, word2, score_cutoff=k)

            if rf > k:
                rf = k
        else:
            with our_timer:
                our = our_unbounded(word1, word2)

            with rf_timer:
                rf = rf_distance(word1, word2)

        assert_eq(our, rf, (word1, word2))

    print(f"{min_length=}, {bounded=}")
    print(f"rapidfuzz:     {rf_timer.ns: 16_} ns")
    print(f"editdistancek: {our_timer.ns: 16_} ns")


if __name__ == "__main__":
    test(0, bounded=False)
    test(100, bounded=False)
    test(1000, bounded=False)
    test(0, bounded=True)
    test(100, bounded=True)
    test(1000, bounded=True)
