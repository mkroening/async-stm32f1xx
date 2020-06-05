set confirm off
set history save on
set pagination off

target remote :3333

monitor arm semihosting enable
monitor reset halt

load
continue
