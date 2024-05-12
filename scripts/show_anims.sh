EXEC="cargo run --bin pop_res --release -- "

DATA=/opt/sandbox/pop/data
FE=${DATA}/fenew

MAIN_PAL_FILE="HSPR0-0.DAT"
MAIN_PAL=/opt/sandbox/pop/data/pal0-0.dat

if [ "$1" == "-s" ]; then
    IDS="61,70,75,81,95,220,221"
    ${EXEC} anims_draw --path ${DATA}/${MAIN_PAL_FILE} --palette ${MAIN_PAL} --composer ul --ids ${IDS} --no_type | magick - - | feh --zoom 300 --force-aliasing -
else
    ${EXEC} anims_draw --path ${DATA}/${MAIN_PAL_FILE} --palette ${MAIN_PAL} --composer ul
fi

#${EXEC} anims_draw --path ${DATA}/${MAIN_PAL_FILE} --palette ${MAIN_PAL} | magick - -crop 640x320+600+0 - | feh --zoom 300 --force-aliasing -
