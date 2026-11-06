"""An optional text model behind the chat route most local model servers share.

No endpoint means names and rows come from data alone, and the stage says so.
"""

from __future__ import annotations

import json
from typing import Protocol

from ..context import Context
from ..errors import StageFailed
from ..net import NetError, Request, Transport
from ..settings import Secret, read_secret


class Chat(Protocol):
    def ask(self, system: str, user: str, max_tokens: int = 16) -> str: ...


class HttpChat:
    def __init__(
        self, endpoint: str, model: str, transport: Transport, key: Secret | None = None
    ) -> None:
        self.endpoint = endpoint
        self.model = model
        self.transport = transport
        self.key = key

    def ask(self, system: str, user: str, max_tokens: int = 16) -> str:
        headers = {"Content-Type": "application/json"}
        if self.key:
            headers["Authorization"] = f"Bearer {self.key.reveal()}"
        body = {
            "model": self.model,
            "temperature": 0,
            "max_tokens": max_tokens,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
        }
        request = Request(
            self.endpoint,
            "/chat/completions",
            headers=headers,
            body=json.dumps(body).encode(),
            timeout=300.0,
        )
        try:
            reply = self.transport.fetch(request)
        except NetError as e:
            raise StageFailed(f"name: the model server gave no answer ({e})") from e
        if reply.status != 200:
            raise StageFailed(f"name: the model server answered {reply.status}")
        return reply.json()["choices"][0]["message"]["content"]


def make_chat(ctx: Context) -> Chat | None:
    """The model client, or None when no endpoint is set."""
    name = ctx.settings.name
    if not name.endpoint:
        return None
    key = read_secret(name.key_file, "name.key_file") if name.key_file else None
    return HttpChat(name.endpoint, name.model, ctx.transport, key)
