#!/usr/bin/env python3
"""
Quicky Notes (.qn) Binary Container <-> JSON Converter & Extractor

Specification:
  [Magic Bytes: 6 bytes (b"QNOTE\\x01")]
  [Meta Length: 4 bytes (u32 little-endian)]
  [JSON Metadata: UTF-8 string of length Meta Length]
  [Contiguous Raw Image Binary Payloads]

Usage Examples:
  # Inspect .qn file and print JSON representation to stdout:
  python3 scripts/qn_to_json.py my_note.qn

  # Convert .qn to .json file (with Base64 embedded images):
  python3 scripts/qn_to_json.py my_note.qn -o note.json

  # Convert .qn to .json file and extract embedded images to an images/ folder:
  python3 scripts/qn_to_json.py my_note.qn -o note.json --extract-images ./images

  # Convert .json file back into a standalone .qn binary file:
  python3 scripts/qn_to_json.py --to-qn note.json -o my_restored_note.qn
"""

import argparse
import base64
import json
import os
import struct
import sys
from pathlib import Path

QN_MAGIC = b"QNOTE\x01"


def decode_qn_file(file_path: str):
    """Decodes a .qn binary file into metadata dict and list of attachment dictionaries."""
    with open(file_path, "rb") as f:
        data = f.read()

    if len(data) < 10:
        raise ValueError(f"File {file_path} is too small to be a valid .qn container (size: {len(data)} bytes)")

    magic = data[0:6]
    if magic != QN_MAGIC:
        raise ValueError(f"Invalid magic signature {magic!r}. Expected {QN_MAGIC!r}")

    (meta_len,) = struct.unpack("<I", data[6:10])
    if len(data) < 10 + meta_len:
        raise ValueError(f"File is truncated: expected at least {10 + meta_len} bytes for header, got {len(data)}")

    meta_json_bytes = data[10 : 10 + meta_len]
    meta = json.loads(meta_json_bytes.decode("utf-8"))

    payload_start = 10 + meta_len
    payload = data[payload_start:]

    images = []
    for img_desc in meta.get("images", []):
        offset = img_desc["offset"]
        length = img_desc["length"]
        if offset + length > len(payload):
            raise ValueError(f"Corrupt payload: image '{img_desc.get('name')}' exceeds file boundary")

        img_bytes = payload[offset : offset + length]
        images.append({
            "id": img_desc["id"],
            "name": img_desc["name"],
            "mime_type": img_desc["mime_type"],
            "size_bytes": len(img_bytes),
            "data": img_bytes,
        })

    return meta, images


def encode_qn_file(meta_dict: dict, attachments: list, output_path: str):
    """Encodes a note dictionary and attachment list into a .qn binary file."""
    image_descriptors = []
    payload_parts = []
    current_offset = 0

    for att in attachments:
        data = att["data"]
        length = len(data)
        image_descriptors.push if hasattr(image_descriptors, "push") else None
        image_descriptors.append({
            "id": att["id"],
            "name": att["name"],
            "mime_type": att.get("mime_type", "image/png"),
            "offset": current_offset,
            "length": length,
        })
        payload_parts.append(data)
        current_offset += length

    meta_payload = {
        "version": meta_dict.get("version", 1),
        "id": meta_dict.get("id", "note-1"),
        "title": meta_dict.get("title", "untitled.qn"),
        "content": meta_dict.get("content", ""),
        "created_at": meta_dict.get("created_at", ""),
        "updated_at": meta_dict.get("updated_at", ""),
        "pinned": meta_dict.get("pinned", False),
        "color_tag": meta_dict.get("color_tag"),
        "images": image_descriptors,
    }

    meta_bytes = json.dumps(meta_payload, ensure_ascii=False).encode("utf-8")
    meta_len = len(meta_bytes)

    with open(output_path, "wb") as f:
        f.write(QN_MAGIC)
        f.write(struct.pack("<I", meta_len))
        f.write(meta_bytes)
        for part in payload_parts:
            f.write(part)


def main():
    parser = argparse.ArgumentParser(
        description="Quicky Notes (.qn) Binary Container <-> JSON Converter & Extractor"
    )
    parser.add_argument("input_file", help="Path to input .qn or .json file")
    parser.add_argument("-o", "--output", help="Output file path (.json or .qn)")
    parser.add_argument(
        "--extract-images",
        metavar="DIR",
        help="Directory to extract all embedded raw image files into",
    )
    parser.add_argument(
        "--to-qn",
        action="store_true",
        help="Convert input JSON file to a .qn binary file",
    )

    args = parser.parse_args()

    input_path = Path(args.input_file)
    if not input_path.exists():
        print(f"Error: File '{input_path}' not found.", file=sys.stderr)
        sys.exit(1)

    if args.to_qn:
        # Convert JSON -> .qn
        with open(input_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        attachments = []
        for att in data.get("attachments", []):
            if "data_base64" in att:
                raw = base64.b64decode(att["data_base64"])
            elif "file_path" in att and os.path.exists(att["file_path"]):
                with open(att["file_path"], "rb") as img_f:
                    raw = img_f.read()
            else:
                raw = b""

            attachments.append({
                "id": att.get("id", "img_1"),
                "name": att.get("name", "image.png"),
                "mime_type": att.get("mime_type", "image/png"),
                "data": raw,
            })

        out_path = args.output or input_path.with_suffix(".qn")
        encode_qn_file(data, attachments, str(out_path))
        print(f"✓ Successfully encoded '{input_path}' -> '{out_path}' with {len(attachments)} attachment(s)")

    else:
        # Convert .qn -> JSON
        try:
            meta, images = decode_qn_file(str(input_path))
        except Exception as e:
            print(f"Error decoding .qn file: {e}", file=sys.stderr)
            sys.exit(1)

        # Extract images if requested
        if args.extract_images:
            extract_dir = Path(args.extract_images).resolve()
            extract_dir.mkdir(parents=True, exist_ok=True)
            for img in images:
                safe_id = "".join(c for c in os.path.basename(str(img.get("id", "img"))) if c.isalnum() or c in "_-")
                safe_name = "".join(c for c in os.path.basename(str(img.get("name", "image.png"))) if c.isalnum() or c in "._-")
                if not safe_name:
                    safe_name = "image.png"
                target_filename = f"{safe_id}_{safe_name}" if safe_id else safe_name
                dest = (extract_dir / target_filename).resolve()
                try:
                    if not dest.is_relative_to(extract_dir):
                        print(f"Warning: Skipping attachment with unsafe traversal path: {target_filename}", file=sys.stderr)
                        continue
                except AttributeError:
                    if not str(dest).startswith(str(extract_dir)):
                        print(f"Warning: Skipping attachment with unsafe traversal path: {target_filename}", file=sys.stderr)
                        continue
                with open(dest, "wb") as f:
                    f.write(img["data"])
                print(f"  Extracted: {dest} ({img['size_bytes']} bytes)")

        # Prepare JSON output
        json_output = {
            "version": meta.get("version", 1),
            "id": meta.get("id", ""),
            "title": meta.get("title", ""),
            "content": meta.get("content", ""),
            "created_at": meta.get("created_at", ""),
            "updated_at": meta.get("updated_at", ""),
            "pinned": meta.get("pinned", False),
            "color_tag": meta.get("color_tag"),
            "attachments": [
                {
                    "id": img["id"],
                    "name": img["name"],
                    "mime_type": img["mime_type"],
                    "size_bytes": img["size_bytes"],
                    "data_base64": base64.b64encode(img["data"]).decode("ascii"),
                }
                for img in images
            ],
        }

        if args.output:
            with open(args.output, "w", encoding="utf-8") as f:
                json.dump(json_output, f, indent=2, ensure_ascii=False)
            print(f"✓ Converted '{input_path}' -> '{args.output}' ({len(images)} images)")
        else:
            print(json.dumps(json_output, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
