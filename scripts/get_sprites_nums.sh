rm -rf ./img_nums
mkdir ./img_nums

EXEC="cargo run --bin pop_res --release -- "

DATA=/opt/sandbox/pop/data
MAIN_PAL_FILE=hfx0-0.dat
MAIN_PAL=/opt/sandbox/pop/data/pal0-0.dat

nums="0x7b 0x7c 0x7a
0x434
0x86 0x8a 0x8f
0x87 0x88 0x89
0x8b 0x8c 0x8d 0x8e
0x90 0x91 0x92 0x93 0x94
0x78 0xa3 0x74 0x76 0x77 0x75 0x85
0xa6 0xa7 0xa4 0xa5 0xa8 0xa9 0x43b 0x79 0x95
0x81 0x82 0x83 0x84
0x49 0x4a 0x4b 0x4c
75
1553 1554 1555 1556 1557
1466
1478 1479 1480 1481 1482 1483 1484 1485 1486 1487 1488
1489 1490 1491 1492 1493 1494 1495 1496 1497 1498 1499
53
11
0x219 0x21a
0x626 0x627 0x628
"

for num in ${nums}; do
    num_dec=$(printf "%u" $num)
    echo "$num/$num_dec"
    ${EXEC} psfb --path ${DATA}/${MAIN_PAL_FILE} --palette ${MAIN_PAL} --num 1 --start ${num_dec} > img_nums/hfx_${num}.bmp
done