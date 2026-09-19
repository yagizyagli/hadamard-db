import sys
import os

# Resolve the absolute workspace root workspace layer directly from this package init file
CURRENT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.abspath(os.path.join(CURRENT_DIR, "../../../"))

# Forcefully append compiler build targets straight into python root search path vectors
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "target/release/maturin"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "target/release"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "core/target/release"))
sys.path.insert(0, os.path.join(WORKSPACE_ROOT, "bindings/python"))

from .client import HadamardClient

__version__ = "0.1.0"
__all__ = ["HadamardClient"]
