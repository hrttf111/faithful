EXEC="cargo run --bin pop_res --release -- "

#rm -rf ./img
mkdir ./img

for i in `seq 0 255`; do
    ${EXEC} water 1 $i > img/water_${i}.bmp
done
