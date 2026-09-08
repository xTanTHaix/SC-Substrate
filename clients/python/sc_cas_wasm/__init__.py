"""
SC-Substrate Python Universal Wasm Client.
High-performance zero-copy buffer protocol bindings.
"""

from .client import SCCASClient, OpCode, EvaluationResult

__all__ = ["SCCASClient", "OpCode", "EvaluationResult"]
