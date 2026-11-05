"""Embedding back ends. The model is a setting; the manifest names whichever one ran."""

from __future__ import annotations

import hashlib
import itertools
import json
import re
from typing import Protocol

import numpy as np

from ..context import Context
from ..errors import StageFailed
from ..net import NetError, Request, Transport
from ..settings import Embed, Secret, need, read_secret

HASHING_MODEL = "hashing-v1"
QUERY_FORM = "Instruct: {task}\nQuery: {text}"
_TOKEN = re.compile(r"[a-z0-9]+")


class Embedder(Protocol):
    model: str

    def encode(self, texts: list[str]) -> np.ndarray: ...

    def encode_queries(self, texts: list[str], task: str, max_length: int) -> np.ndarray: ...


class HashingEmbedder:
    """Signed feature hashing of words and word pairs. It needs no model, no download and no GPU.

    It proves function on a laptop. It knows nothing about meaning, so its scores say nothing about
    row quality.
    """

    model = HASHING_MODEL

    def __init__(self, dimensions: int) -> None:
        self.dimensions = dimensions

    def _one(self, text: str) -> np.ndarray:
        words = _TOKEN.findall(text.lower())
        grams = words + [f"{a} {b}" for a, b in itertools.pairwise(words)]
        v = np.zeros(self.dimensions, dtype=np.float32)
        for gram in grams:
            digest = hashlib.blake2b(gram.encode(), digest_size=8).digest()
            value = int.from_bytes(digest, "little")
            v[(value >> 1) % self.dimensions] += 1.0 if value & 1 else -1.0
        if not grams or not np.any(v):
            v[0] = 1.0
        return v

    def encode(self, texts: list[str]) -> np.ndarray:
        return np.stack([self._one(t) for t in texts])

    def encode_queries(self, texts: list[str], task: str, max_length: int) -> np.ndarray:
        return self.encode(texts)


class LocalModelEmbedder:
    """A sentence-transformers model on this machine, for full builds on a GPU."""

    def __init__(self, model: str, device: str, batch_size: int, max_length: int) -> None:
        try:
            from sentence_transformers import SentenceTransformer
        except ImportError as e:
            raise StageFailed("embed: install the local-model extra for this back end") from e
        self.model = model
        self.batch_size = batch_size
        self.max_length = max_length
        self.net = SentenceTransformer(model, device=device or None)

    def _run(self, texts: list[str], batch_size: int) -> np.ndarray:
        return self.net.encode(
            texts, batch_size=batch_size, normalize_embeddings=False, show_progress_bar=False
        ).astype(np.float32)

    def encode(self, texts: list[str]) -> np.ndarray:
        self.net.max_seq_length = self.max_length
        return self._run(texts, self.batch_size)

    def encode_queries(self, texts: list[str], task: str, max_length: int) -> np.ndarray:
        self.net.max_seq_length = max_length
        queries = [QUERY_FORM.format(task=task, text=t) for t in texts]
        return self._run(queries, 32)


class HttpEmbedder:
    """A server that speaks the embeddings route most local model servers share."""

    def __init__(
        self, endpoint: str, model: str, transport: Transport, key: Secret | None = None
    ) -> None:
        self.endpoint = endpoint
        self.model = model
        self.transport = transport
        self.key = key

    def encode(self, texts: list[str]) -> np.ndarray:
        headers = {"Content-Type": "application/json"}
        if self.key:
            headers["Authorization"] = f"Bearer {self.key.reveal()}"
        body = json.dumps({"model": self.model, "input": texts}).encode()
        try:
            reply = self.transport.fetch(
                Request(self.endpoint, "/embeddings", headers=headers, body=body, timeout=600.0)
            )
        except NetError as e:
            raise StageFailed(f"embed: the model server gave no answer ({e})") from e
        if reply.status != 200:
            raise StageFailed(f"embed: the model server answered {reply.status}")
        rows = sorted(reply.json()["data"], key=lambda d: d["index"])
        return np.array([r["embedding"] for r in rows], dtype=np.float32)

    def encode_queries(self, texts: list[str], task: str, max_length: int) -> np.ndarray:
        return self.encode([QUERY_FORM.format(task=task, text=t) for t in texts])


def make_embedder(ctx: Context, embed: Embed | None = None) -> Embedder:
    e = embed or ctx.settings.embed
    if e.backend == "hashing":
        return HashingEmbedder(e.dimensions)
    if e.backend == "local-model":
        return LocalModelEmbedder(
            need(e.model, "embed.model"), e.device, e.batch_size, e.max_length
        )
    if e.backend == "http":
        key = read_secret(e.key_file, "embed.key_file") if e.key_file else None
        return HttpEmbedder(
            need(e.endpoint, "embed.endpoint"), need(e.model, "embed.model"), ctx.transport, key
        )
    raise StageFailed(f"embed: unknown back end {e.backend!r}")
