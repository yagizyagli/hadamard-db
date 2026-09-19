"""
Hadamard-DB: Enterprise-grade, ultra-lightweight quantum database query engine.
Copyright 2026 Yağız Yağlı. Licensed under Apache-2.0.
"""

import sys
import os

# 1. ALWAYS RESOLVE WORKSPACE PATHS FIRST (Ensure alignment with core client structures)
CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.abspath(os.path.join(CURRENT_DIR, "../../../"))

# 2. FORCEFULLY APPEND COMPILER BUILD TARGETS INTO ROOT Python SEARCH VECTORS
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "target/release/maturin"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "target/release"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "core/target/release"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "bindings/python"))

# 3. NOW EXPOSE THE CLIENT EXVELOPE SAFELY TO THE PYTHON ECOSYSTEM
from .client import HadamardClient

__version__ = "0.1.0"
__all__ = ["HadamardClient"]
