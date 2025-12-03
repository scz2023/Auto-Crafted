# Python Runtime

This directory contains the bundled Python 3.13.0 interpreter.

## Directory Structure

python/ - Python interpreter files
  python.exe - Python executable
  python*.dll - Python dynamic link libraries
  python*.pth - Python path configuration files
  get-pip.py - pip installation script (optional)

## Install pip (Optional)

If you need to use pip to install packages, you can run:

python-runtime/python/python.exe python-runtime/python/get-pip.py

## Notes

1. This is an embedded Python, does not include the full standard library
2. Some packages that require compilation may not be directly installable
3. It is recommended to use pre-compiled wheel packages

## Version Information

Python Version: 3.13.0
Platform: windows-x64
Packaged Time: 2025-12-04 02:36:05
