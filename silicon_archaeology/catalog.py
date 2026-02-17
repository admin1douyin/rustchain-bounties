#!/usr/bin/env python3
"""
Software Asset Cataloger for silicon-archaeology-skill

Produces SHA-256 + SHA-512 fixity hashes and manifest JSON.
Compatible with Echoes paper manifest format.

Usage:
    python3 catalog.py <file_or_directory>
    python3 catalog.py --batch <directory>
"""

import hashlib
import json
import os
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional


def calculate_sha256(filepath: str) -> str:
    """Calculate SHA-256 hash of a file."""
    sha256 = hashlib.sha256()
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(8192), b''):
            sha256.update(chunk)
    return sha256.hexdigest()


def calculate_sha512(filepath: str) -> str:
    """Calculate SHA-512 hash of a file."""
    sha512 = hashlib.sha512()
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(8192), b''):
            sha512.update(chunk)
    return sha512.hexdigest()


def get_file_info(filepath: str) -> Dict:
    """Get file information for manifest."""
    stat = os.stat(filepath)
    return {
        'filename': os.path.basename(filepath),
        'size': stat.st_size,
        'sha256': calculate_sha256(filepath),
        'sha512': calculate_sha512(filepath),
        'description': '',  # To be filled by user
        'epoch': int(time.time()),
        'cataloged_at': datetime.utcnow().isoformat() + 'Z'
    }


def catalog_file(filepath: str, description: str = '') -> Dict:
    """Catalog a single file."""
    info = get_file_info(filepath)
    if description:
        info['description'] = description
    return info


def catalog_directory(directory: str, recursive: bool = True) -> List[Dict]:
    """Catalog all files in a directory."""
    manifest = []
    path = Path(directory)
    
    pattern = '**/*' if recursive else '*'
    for filepath in path.glob(pattern):
        if filepath.is_file():
            manifest.append(catalog_file(str(filepath)))
    
    return manifest


def save_manifest(manifest: Dict, output_path: str = 'manifest.json'):
    """Save manifest to JSON file."""
    with open(output_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    print(f"Manifest saved to {output_path}")


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print("Usage: python3 catalog.py <file_or_directory> [--batch]")
        print("       python3 catalog.py --batch <directory>")
        sys.exit(1)
    
    path = sys.argv[1]
    batch_mode = '--batch' in sys.argv or '-b' in sys.argv
    
    if not os.path.exists(path):
        print(f"Error: Path does not exist: {path}")
        sys.exit(1)
    
    if os.path.isfile(path):
        manifest = catalog_file(path)
        print(json.dumps(manifest, indent=2))
        save_manifest(manifest)
    elif os.path.isdir(path):
        if batch_mode:
            manifest = catalog_directory(path, recursive=True)
        else:
            manifest = catalog_directory(path, recursive=False)
        print(f"Cataloged {len(manifest)} files")
        save_manifest({'assets': manifest, 'total': len(manifest)})
    else:
        print(f"Error: Invalid path: {path}")
        sys.exit(1)


if __name__ == '__main__':
    main()
