import sys
from pathlib import Path
from ctypes import *

MY_DIR = Path(__file__).parent.resolve()

BUILD_TYPE = "release" if "--release" in sys.argv else "debug"

DLL_PATH = MY_DIR / "target" / BUILD_TYPE / "pathfinder_c_api_fun.dll"

DLL_STR = str(DLL_PATH)

print(f"Size of DLL is {DLL_PATH.stat().st_size} bytes.")

cdll.LoadLibrary(DLL_STR)

fun = CDLL(DLL_STR)

result = fun.boop(2)

print(f"result is {result}.")

assert result == 642

print("Hooray, it works!")
