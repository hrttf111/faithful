rm -rf ./img
mkdir ./img
#DAT=/opt/sandbox/pop/data/hfx0-0.dat
DAT=/opt/sandbox/pop/data/HSPR0-0.DAT
#DAT=/opt/sandbox/pop/data/fenew/FEhi33EE.spr
#DAT=/opt/sandbox/pop/data/EDIT0-0.DAT
#DAT=/opt/sandbox/pop/data/FONT2-0.DAT
#DAT=/opt/sandbox/pop/data/fenew/fecursor.spr
#DAT=/opt/sandbox/pop/data/plspanel.spr
#DAT=/opt/sandbox/pop/data/POINT0-0.DAT
#DAT=/opt/sandbox/pop/data/plsspace.spr
#DAT=/opt/sandbox/pop/data/f00t3-0.dat
#for i in `seq 0 7952`; do
#    cargo run --bin pop_res --release -- psfb --path ${DAT} --palette /opt/sandbox/pop/data/pal0-0.dat --num=$i > img/$i.bmp
#done
cargo run --bin pop_res --release -- psfb --path ${DAT} --palette /opt/sandbox/pop/data/pal0-0.dat --prefix /opt/projects/faithful/img/hspr0_
#cargo run --bin pop_res --release -- psfb --path ${DAT} --info
#cargo run --bin pop_res --release -- psfb --path ${DAT} --palette /opt/sandbox/pop/data/fenew/fepal0.dat --prefix /opt/projects/faithful/img/hspr0_

#cargo run --bin pop_res --release -- psfb --path ${DAT} --palette /opt/sandbox/pop/data/fenew/fepal0.dat --prefix /opt/projects/faithful/img/hspr0_

# cargo run --bin pop_res -- psfb --path /opt/sandbox/pop/data/./fenew/felgspen.spr --num=0 | feh --zoom 500 --force-aliasing
# cargo run --bin pop_res -- psfb --path /opt/sandbox/pop/data/./fenew/felgspen.spr --palette /opt/sandbox/pop/data/fenew/fepal0.dat --num=0 | feh --zoom 500 --force-aliasing
#--palette /opt/sandbox/pop/data/pal0-0.dat
