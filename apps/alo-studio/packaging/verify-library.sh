#!/usr/bin/env bash
set -euo pipefail
scratch=${1:?Pass the isolated UI fixture}
python3 - "$XDG_DATA_HOME/alo-studio/recordings" "$scratch" <<'PY'
import json,os,pathlib,struct,sys,zlib
folder=pathlib.Path(sys.argv[1]);scratch=pathlib.Path(sys.argv[2])
entries=list(folder.glob('*.json'))
assert len(entries)==1,'The GUI fixture must first save one real recording'
clip=json.loads(entries[0].read_text());source=folder/(clip['id']+'.webm')
assert source.stat().st_size>1024
def chunk(kind,data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data))
compressor=zlib.compressobj();encoded=[]
row=b'\0'+bytes([40,80,120,255])*4096
for _ in range(4096):encoded.append(compressor.compress(row))
encoded.append(compressor.flush())
thumbnail=scratch/'library-preview.png'
thumbnail.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',4096,4096,8,6,0,0,0))+chunk(b'IDAT',b''.join(encoded))+chunk(b'IEND',b''))
for index in range(128):
    item=dict(clip,id=f"1794000000-{index}-1",title=f"Library recording {index}")
    (folder/(item['id']+'.json')).write_text(json.dumps(item))
    os.link(source,folder/(item['id']+'.webm'))
    os.link(thumbnail,folder/(item['id']+'.png'))
PY
win=$(xdotool search --onlyvisible --name '^alo Studio$' | head -1)
xdotool windowfocus "$win"
xdotool key ctrl+l
sleep .2
bus=$(cat "$scratch/session-bus")
gdbus call --address "$bus" --timeout 2 --dest org.aloworld.Studio --object-path /org/aloworld/Studio/Recorder --method org.aloworld.Studio.Recorder1.GetStatus > "$scratch/library-responsive-status.txt"
xdotool key ctrl+n
sleep .2
gdbus call --address "$bus" --timeout 2 --dest org.aloworld.Studio --object-path /org/aloworld/Studio/Recorder --method org.aloworld.Studio.Recorder1.GetStatus > "$scratch/library-dismissed-status.txt"
grep -q '"phase":"ready"' "$scratch/library-dismissed-status.txt"
printf 'PASS: Library controls stay responsive while 128 large previews load and when dismissed\n'
