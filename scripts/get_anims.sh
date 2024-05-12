#rm -rf ./img
#mkdir ./img

EXEC="cargo run --bin pop_res --release -- "

DATA=/opt/sandbox/pop/data
FE=${DATA}/fenew

MAIN_PAL_FILE="HSPR0-0.DAT"
MAIN_PAL=/opt/sandbox/pop/data/pal0-0.dat

#${EXEC} anims_draw --path ${DATA}/${MAIN_PAL_FILE} --palette ${MAIN_PAL} > img/anims.bmp
${EXEC} anims --psfb_path ${DATA}/${MAIN_PAL_FILE} > 1
