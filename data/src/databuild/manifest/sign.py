"""Ed25519 signatures over the exact bytes of a manifest. The key never prints."""

from __future__ import annotations

import base64
import binascii
import hashlib
import json
import os
from pathlib import Path
from typing import Any

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey

from ..errors import StageFailed
from ..settings import Secret

ALGORITHM = "ed25519"


def _raw(public: Ed25519PublicKey) -> bytes:
    return public.public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw)


def key_id(public: Ed25519PublicKey) -> str:
    return hashlib.sha256(_raw(public)).hexdigest()[:16]


def read_private(secret: Secret) -> Ed25519PrivateKey:
    try:
        key = serialization.load_pem_private_key(secret.reveal().encode(), password=None)
    except (ValueError, TypeError) as e:
        raise StageFailed("manifest: the signing key cannot be read") from e
    if not isinstance(key, Ed25519PrivateKey):
        raise StageFailed("manifest: the signing key is not an ed25519 key")
    return key


def read_public(path: str) -> Ed25519PublicKey:
    try:
        key = serialization.load_pem_public_key(Path(path).read_bytes())
    except OSError as e:
        raise StageFailed(f"manifest: cannot read the public key: {e.strerror}") from e
    except ValueError as e:
        raise StageFailed("manifest: the public key cannot be read") from e
    if not isinstance(key, Ed25519PublicKey):
        raise StageFailed("manifest: the public key is not an ed25519 key")
    return key


def sign(data: bytes, key: Ed25519PrivateKey) -> dict[str, str]:
    return {
        "algorithm": ALGORITHM,
        "key_id": key_id(key.public_key()),
        "signature": base64.b64encode(key.sign(data)).decode(),
    }


def check_signature(data: bytes, signature: dict[str, Any], public: Ed25519PublicKey) -> None:
    """Refuses a signature made by another key or over other bytes."""
    if signature.get("key_id") != key_id(public):
        raise StageFailed("manifest: signed by a key this check does not hold")
    try:
        public.verify(base64.b64decode(signature["signature"], validate=True), data)
    except (InvalidSignature, binascii.Error, ValueError) as e:
        raise StageFailed("manifest: the signature does not match the manifest") from e


def write_keypair(private_path: Path, public_path: Path) -> str:
    """Makes a key pair and returns its id. An existing key file is never overwritten."""
    for path in (private_path, public_path):
        if path.exists():
            raise StageFailed(f"keygen: {path} exists")
    key = Ed25519PrivateKey.generate()
    pem = key.private_bytes(
        serialization.Encoding.PEM, serialization.PrivateFormat.PKCS8, serialization.NoEncryption()
    )
    descriptor = os.open(private_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "wb") as out:
        out.write(pem)
    public = key.public_key()
    public_path.write_bytes(
        public.public_bytes(
            serialization.Encoding.PEM, serialization.PublicFormat.SubjectPublicKeyInfo
        )
    )
    return key_id(public)


def signature_bytes(signature: dict[str, str]) -> bytes:
    return (json.dumps(signature, indent=1, sort_keys=True) + "\n").encode()
