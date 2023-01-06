rm -rf ./img
mkdir ./img

for i in `seq 0 255`; do
    cargo run --bin pop_res --release -- water 1 $i > img/water_${i}.bmp
done
