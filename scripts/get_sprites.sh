rm -rf ./img
mkdir ./img

EXEC="cargo run --bin pop_res --release -- "

DATA=/opt/sandbox/pop/data
FE=${DATA}/fenew

MAIN_PAL_FILES="
hfx0-0.dat
HSPR0-0.DAT
EDIT0-0.DAT
FONT2-0.DAT
POINT0-0.DAT
f00t3-0.dat
"
MAIN_PAL=/opt/sandbox/pop/data/pal0-0.dat

for file in $MAIN_PAL_FILES; do
    ${EXEC} psfb --path ${DATA}/${file} --palette ${MAIN_PAL} > img/${file}.bmp
done

FE_PAL_FILES="
FEhi33EE.spr
fecursor.spr
"

for file in $FE_PAL_FILES; do
    ${EXEC} psfb --path ${FE}/${file} --palette ${MAIN_PAL} > img/${file}.bmp
done

# cargo run --bin pop_res -- psfb --path /opt/sandbox/pop/data/./fenew/felgspen.spr --palette /opt/sandbox/pop/data/fenew/fepal0.dat --num=0 | feh --zoom 500 --force-aliasing
